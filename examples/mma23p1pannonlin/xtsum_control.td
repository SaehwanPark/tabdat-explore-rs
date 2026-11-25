# Independent Base R control: uneven panels and variable-specific missingness.
use reference/xtsum_unbalanced_input.csv
xtsum x y, i(firm)
