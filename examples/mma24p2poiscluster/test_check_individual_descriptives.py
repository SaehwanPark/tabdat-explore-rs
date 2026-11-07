#!/usr/bin/env python3
"""Real-script controls and targeted source/sample/statistic artifact corruption checks."""
import csv
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from check_individual_descriptives import audit, rows

ROOT=Path(__file__).resolve().parents[2]
FIXTURES=ROOT/'tests/fixtures/mma24_individual_descriptives'
SCRIPT=ROOT/'examples/mma24p2poiscluster/individual_descriptives.td'
BIN=ROOT/'target/debug/tabdat-explore-rs'


def write_rows(path, records):
  with path.open('w',newline='') as f:
    writer=csv.DictWriter(f,fieldnames=list(records[0]))
    writer.writeheader();writer.writerows(records)


def run(native):
  result=subprocess.run([str(BIN),'--json','-f',str(SCRIPT)],cwd=native,capture_output=True,text=True)
  assert result.returncode==0,result.stderr
  (native/'result.jsonl').write_text(result.stdout)


class IndividualDescriptiveAudit(unittest.TestCase):
  def test_controls_and_permutation(self):
    with tempfile.TemporaryDirectory() as tmp:
      root=Path(tmp)
      for case in ('missing','blank'):
        native=root/case;native.mkdir()
        shutil.copyfile(FIXTURES/case/'input.csv',native/'vietnam_individual.csv');run(native)
        self.assertEqual(audit(native,FIXTURES/case,FIXTURES/case/'input.csv','--csv'),(8 if case=='missing' else 1,60,0))
      native=root/'permuted';native.mkdir()
      raw=list(reversed(rows(FIXTURES/'missing/input.csv')))
      write_rows(native/'vietnam_individual.csv',raw)
      reference=root/'reference';shutil.copytree(FIXTURES/'missing',reference)
      independent={r['source_row']:r for r in rows(reference/'source.csv')}
      write_rows(reference/'source.csv',[independent[r['source_row']] for r in raw])
      run(native)
      self.assertEqual(audit(native,reference,native/'vietnam_individual.csv','--csv'),(8,60,0))
      for name in ('individual_descriptive_source.csv','individual_descriptives.csv'):
        self.assertEqual((native/name).read_bytes(),(root/'missing'/name).read_bytes())

  def test_twenty_forgeries(self):
    with tempfile.TemporaryDirectory() as tmp:
      root=Path(tmp);native=root/'baseline';native.mkdir()
      shutil.copyfile(FIXTURES/'missing/input.csv',native/'vietnam_individual.csv');run(native)
      self.assertEqual(audit(native,FIXTURES/'missing',FIXTURES/'missing/input.csv','--csv'),(8,60,0))
      for forgery in range(20):
        with self.subTest(forgery=forgery):
          target=root/f'native-{forgery}';reference=root/f'reference-{forgery}'
          shutil.copytree(native,target);shutil.copytree(FIXTURES/'missing',reference)
          events=[json.loads(line) for line in (target/'result.jsonl').read_text().splitlines()]
          prepared=rows(target/'individual_descriptive_source.csv');report=rows(target/'individual_descriptives.csv')
          if forgery==0:
            source=rows(target/'vietnam_individual.csv');source[1]['age']='4.5';write_rows(target/'vietnam_individual.csv',source)
          elif forgery==1: prepared=[r for r in prepared if r['source_row']!='6']
          elif forgery==2: prepared[0],prepared[1]=prepared[1],prepared[0]
          elif forgery==3: prepared[0]['age']='2.5'
          elif forgery==4: events[4]['data']['rows'][0]['mean']+=.25
          elif forgery==5: report[0]['mean']='3'
          elif forgery==6:
            oracle=rows(reference/'summary.csv');oracle[0]['mean']='3';write_rows(reference/'summary.csv',oracle)
            report[0]['mean']='3';events[4]['data']['rows'][0]['mean']=3
          elif forgery==7: events[4]['data']['rows'][1]['count']=5
          elif forgery==8: report[0],report[1]=report[1],report[0]
          elif forgery==9:
            members=rows(reference/'members.csv');members[0]['source_row']='2';write_rows(reference/'members.csv',members)
          elif forgery==10: events[4]['data']['rows'][5]['mean']=0
          elif forgery==11: events[4]['data']['rows'][7]['std_dev']=0
          elif forgery==12: events[4]['data']['rows'][0]['std_dev']*= (4/5)**.5
          elif forgery==13: events[-1]['data']['table_name']='individual_descriptives'
          elif forgery==14: events[0]['data']['dataset']['row_count']=7
          elif forgery==15: events[3]['data']['dataset']['columns'][1]['data_type']='DOUBLE'
          elif forgery==16: events[4]['result_type']='TableResult'
          elif forgery==17: events[4]['schema_version']=2
          elif forgery==18: events[2]['data']['rows'][0][1]=0
          elif forgery==19: prepared[5]['commune']='1'
          write_rows(target/'individual_descriptive_source.csv',prepared)
          write_rows(target/'individual_descriptives.csv',report)
          (target/'result.jsonl').write_text(''.join(json.dumps(e)+'\n' for e in events))
          with self.assertRaises((AssertionError,ValueError,KeyError,IndexError)):
            audit(target,reference,FIXTURES/'missing/input.csv','--csv')


if __name__=='__main__':
  unittest.main()
