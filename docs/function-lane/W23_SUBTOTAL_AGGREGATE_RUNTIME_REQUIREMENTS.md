# W23 Runtime Requirements - SUBTOTAL / AGGREGATE

1. manual-hidden block:
   - on its own worksheet
   - `A2 := 10`
   - `A3 := 20`
   - `A4 := =SUBTOTAL(9,A2:A3)`
   - `A5 := 40`
   - row `3` manually hidden
2. filtered block:
   - on its own worksheet
   - `D9 := vals`
   - `D10 := 10`
   - `D11 := 20`
   - `D12 := =SUBTOTAL(9,D10:D11)`
   - `D13 := 40`
   - filter excludes `20`
3. hidden-plus-error block:
   - on its own worksheet
   - `H20 := 10`
   - `H21 := 20`
   - `H22 := =SUBTOTAL(9,H20:H21)`
   - `H23 := 40`
   - `H24 := =NA()`
   - row `21` manually hidden
4. large/small style block:
   - on its own worksheet
   - `K30:K34 := 1,9,3,7,5`
   - row `31` manually hidden
5. normalize COM sentinel `-2146826246` to `#N/A`
