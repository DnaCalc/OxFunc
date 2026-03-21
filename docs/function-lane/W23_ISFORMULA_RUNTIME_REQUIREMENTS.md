# W23 Runtime Requirements - ISFORMULA

1. workbook with:
   - `A1 := =1+2`
   - `A2 := plain`
   - `A3 := =""`
2. evaluate manifest formulas in a separate output column
3. normalize COM error sentinel `-2146826273` to `#VALUE!`
4. compare worksheet text to manifest expected text
