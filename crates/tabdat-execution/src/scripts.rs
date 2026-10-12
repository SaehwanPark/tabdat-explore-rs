use crate::*;

impl Session {
  /// Execute a .td script file against this session.
  pub fn execute_run(
    &mut self,
    raw_path: impl AsRef<Path>,
  ) -> Result<ExecutionResult, RuntimeError> {
    let result = self.execute_run_with_callbacks(raw_path, |_| Ok(()), |_| {})?;
    Ok(ExecutionResult::Run(result))
  }
  /// Execute a `.td` script while validating commands before execution and
  /// observing each successful leaf-command result.
  ///
  /// Callbacks run in script execution order, including nested scripts. A
  /// validation error is attached to the command's script path and line.
  pub fn execute_run_with_callbacks<Before, Observe>(
    &mut self,
    raw_path: impl AsRef<Path>,
    mut before_command: Before,
    mut observe_result: Observe,
  ) -> Result<RunResult, RuntimeError>
  where
    Before: FnMut(&Command) -> Result<(), String>,
    Observe: FnMut(&ExecutionResult),
  {
    let mut active_stack = Vec::new();
    let mut context = ScriptContext::empty();
    self.execute_script_file(
      raw_path.as_ref(),
      None,
      &mut active_stack,
      &mut context,
      &mut before_command,
      &mut observe_result,
    )
  }
  pub(crate) fn execute_script_file<Before, Observe>(
    &mut self,
    path: &Path,
    base_dir: Option<&Path>,
    active_stack: &mut Vec<PathBuf>,
    context: &mut ScriptContext,
    before_command: &mut Before,
    observe_result: &mut Observe,
  ) -> Result<RunResult, RuntimeError>
  where
    Before: FnMut(&Command) -> Result<(), String>,
    Observe: FnMut(&ExecutionResult),
  {
    let resolved_path = resolve_script_path(path, base_dir);
    if active_stack.contains(&resolved_path) {
      return Err(RuntimeError::ScriptError(ScriptError::new(
        resolved_path,
        1,
        "recursive script inclusion is not supported",
      )));
    }

    let commands = read_script(&resolved_path)?;
    active_stack.push(resolved_path.clone());

    let mut block_state: Option<ScriptBlockState> = None;
    let mut executed_commands = 0;

    for script_command in commands {
      let line = script_command.start_line;
      let raw_text = script_command.text;

      // In an inactive branch, only control-flow directives are evaluated.
      if let Some(ref state) = block_state
        && !state.current_branch_active()
      {
        let stripped = raw_text.trim();
        let first_token = stripped
          .split_whitespace()
          .next()
          .unwrap_or("")
          .to_ascii_lowercase();
        if first_token != "if" && first_token != "else" && first_token != "end" {
          continue;
        }
      }

      let expanded_text = expand_script_macros(&raw_text, context, &resolved_path, line)?;

      // Control flow directives: if, else, end
      if let Some(control) = parse_control_flow_directive(&expanded_text, &resolved_path, line)? {
        match control {
          ControlFlowDirective::If(directive) => {
            if block_state.is_some() {
              return Err(RuntimeError::ScriptError(ScriptError::new(
                &resolved_path,
                line,
                "nested if blocks are not supported",
              )));
            }
            block_state = Some(ScriptBlockState::new(line, directive.active));
          }
          ControlFlowDirective::Else(_) => {
            let Some(ref mut state) = block_state else {
              return Err(RuntimeError::ScriptError(ScriptError::new(
                &resolved_path,
                line,
                "else without matching if",
              )));
            };
            if state.in_else {
              return Err(RuntimeError::ScriptError(ScriptError::new(
                &resolved_path,
                line,
                "if block already has an else branch",
              )));
            }
            state.in_else = true;
          }
          ControlFlowDirective::End(_) => {
            if block_state.is_none() {
              return Err(RuntimeError::ScriptError(ScriptError::new(
                &resolved_path,
                line,
                "end without matching if",
              )));
            }
            block_state = None;
          }
        }
        continue;
      }

      // Script directives: seed, let
      if let Some(directive) =
        parse_script_directive(&expanded_text, context, &resolved_path, line)?
      {
        match directive {
          ScriptDirective::Seed(seed_dir) => {
            context.seed = Some(seed_dir.value);
          }
          ScriptDirective::Let(let_dir) => {
            context.macros.insert(let_dir.name, let_dir.value);
          }
        }
        continue;
      }

      // Parse and execute command
      let command = match tabdat_language::parse_command(&expanded_text) {
        Ok(cmd) => cmd,
        Err(err) => {
          return Err(RuntimeError::ScriptError(ScriptError::new(
            &resolved_path,
            line,
            err.to_string(),
          )));
        }
      };

      if let Err(message) = before_command(&command) {
        return Err(RuntimeError::ScriptError(ScriptError::new(
          &resolved_path,
          line,
          message,
        )));
      }

      match command {
        Command::Exit => {
          break;
        }
        Command::Run { path: nested_path } => {
          let nested_base = resolved_path.parent();
          let nested_result = self.execute_script_file(
            Path::new(&nested_path),
            nested_base,
            active_stack,
            context,
            before_command,
            observe_result,
          )?;
          executed_commands += nested_result.executed_commands;
        }
        other => match self.execute(other) {
          Ok(result) => {
            executed_commands += 1;
            observe_result(&result);
          }
          Err(RuntimeError::ScriptError(err)) => {
            return Err(RuntimeError::ScriptError(err));
          }
          Err(err) => {
            return Err(RuntimeError::ScriptError(ScriptError::new(
              &resolved_path,
              line,
              err.to_string(),
            )));
          }
        },
      }
    }

    if let Some(state) = block_state {
      return Err(RuntimeError::ScriptError(ScriptError::new(
        &resolved_path,
        state.start_line,
        "if block is missing end",
      )));
    }

    active_stack.pop();

    Ok(RunResult {
      path: resolved_path,
      executed_commands,
    })
  }
}
