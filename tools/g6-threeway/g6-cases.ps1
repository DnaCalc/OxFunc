# G6 three-way discrepancy witness set.
#
# Every open catalog G6 (financial) discrepancy maps to a function implemented in
# the public ExcelFinancialFunctions F# library (Luca Bolognese). This file is the
# single source of truth for the witnesses; run-g6-threeway.ps1 renders each case
# into all three engines (OxFunc local-eval, F# FSI, live Excel COM) bit-exactly.
#
# Neutral typed args (identical order across Excel / F# / OxFunc):
#   @{ t='num'    ; v=<double> }   plain numeric (rate, par, price, ...)
#   @{ t='date'   ; v=<serial> }   Excel date serial -> F# DateTime.FromOADate
#   @{ t='freq'   ; v=<1|2|4> }    -> F# enum<Frequency>
#   @{ t='basis'  ; v=<0..4> }     -> F# enum<DayCountBasis>
#   @{ t='paydue' ; v=<0|1> }      -> F# enum<PaymentDue>
#   @{ t='array'  ; v=@(...) }     numeric vector (IRR)
#
# Per case: id, row (catalog row), xlFn (Excel name), fsFn (F# member),
# residual (known OxFunc-vs-Excel gap from the catalog), args.

$epoch = [datetime]'1899-12-30'
function S([string]$d) { [int]([datetime]$d - $epoch).Days }

$G6Cases = @()

# --- PMT / PPMT (IPMT adjacent) : annuity publication exactness ----------------
$rate512 = 0.05 / 12
$G6Cases += @{ id='pmt.witness'; row='PMT/PPMT'; xlFn='PMT'; fsFn='Pmt';
  residual='PMT(0.05/12,360,200000) ~8 ULP';
  args=@(@{t='num';v=$rate512},@{t='num';v=360},@{t='num';v=200000},@{t='num';v=0},@{t='paydue';v=0}) }
$G6Cases += @{ id='ppmt.witness'; row='PMT/PPMT'; xlFn='PPMT'; fsFn='PPmt';
  residual='PPMT(0.05/12,1,360,200000) ~63 ULP';
  args=@(@{t='num';v=$rate512},@{t='num';v=1},@{t='num';v=360},@{t='num';v=200000},@{t='num';v=0},@{t='paydue';v=0}) }
$G6Cases += @{ id='ipmt.adjacent'; row='PMT/PPMT'; xlFn='IPMT'; fsFn='IPmt';
  residual='IPMT adjacent to PPMT publication path';
  args=@(@{t='num';v=$rate512},@{t='num';v=1},@{t='num';v=360},@{t='num';v=200000},@{t='num';v=0},@{t='paydue';v=0}) }

# --- ACCRINT : quasi-coupon summation; act/act residual ------------------------
$G6Cases += @{ id='accrint.witness.b0'; row='ACCRINT'; xlFn='ACCRINT'; fsFn='AccrInt';
  residual='30/360 witness (now bit-exact 25)';
  args=@(@{t='date';v=(S '2020-01-01')},@{t='date';v=(S '2021-01-01')},@{t='date';v=(S '2020-07-01')},
         @{t='num';v=0.05},@{t='num';v=1000},@{t='freq';v=2},@{t='basis';v=0}) }
$G6Cases += @{ id='accrint.actact.multi'; row='ACCRINT'; xlFn='ACCRINT'; fsFn='AccrInt';
  residual='act/act (basis 1) multi-coupon span crossing leap Feb ~0.07%';
  args=@(@{t='date';v=(S '2019-01-01')},@{t='date';v=(S '2021-01-01')},@{t='date';v=(S '2020-07-01')},
         @{t='num';v=0.05},@{t='num';v=1000},@{t='freq';v=2},@{t='basis';v=1}) }

# --- YIELD : solver residual ~19 ULP ------------------------------------------
$G6Cases += @{ id='yield.witness'; row='YIELD'; xlFn='YIELD'; fsFn='Yield';
  residual='~19 ULP (bisection vs Excel solver)';
  args=@(@{t='date';v=44013},@{t='date';v=44562},@{t='num';v=0.05},@{t='num';v=95},
         @{t='num';v=100},@{t='freq';v=2},@{t='basis';v=0}) }

# --- ODDFPRICE / ODDFYIELD : 13-case settlement x basis x Nq matrix -----------
# ODDFPRICE(settlement, maturity, issue, first_coupon, rate, yld, redemption, frequency, basis)
function PriceCase($id, $settle, $mat, $issue, $first, $basis) {
  @{ id=$id; row='ODDFPRICE/ODDFYIELD'; xlFn='ODDFPRICE'; fsFn='OddFPrice'; residual='30/360 ~1 ULP; actual bases not yet bit-exact';
     args=@(@{t='date';v=(S $settle)},@{t='date';v=(S $mat)},@{t='date';v=(S $issue)},@{t='date';v=(S $first)},
            @{t='num';v=0.05},@{t='num';v=0.06},@{t='num';v=100},@{t='freq';v=2},@{t='basis';v=$basis}) }
}
function YieldCase($id, $settle, $mat, $issue, $first, $basis) {
  @{ id=$id; row='ODDFPRICE/ODDFYIELD'; xlFn='ODDFYIELD'; fsFn='OddFYield'; residual='inverts ODDFPRICE';
     args=@(@{t='date';v=(S $settle)},@{t='date';v=(S $mat)},@{t='date';v=(S $issue)},@{t='date';v=(S $first)},
            @{t='num';v=0.05},@{t='num';v=95},@{t='num';v=100},@{t='freq';v=2},@{t='basis';v=$basis}) }
}
$G6Cases += (PriceCase 'oddfprice.witness' '2020-07-01' '2022-01-01' '2020-01-01' '2021-01-01' 0)
$G6Cases += (PriceCase 'oddfprice.mid2'    '2020-09-15' '2022-01-01' '2020-01-01' '2021-01-01' 0)
$G6Cases += (PriceCase 'oddfprice.in1'     '2020-03-15' '2022-01-01' '2020-01-01' '2021-01-01' 0)
$G6Cases += (PriceCase 'oddfprice.b1'      '2020-09-15' '2022-01-01' '2020-01-01' '2021-01-01' 1)
$G6Cases += (PriceCase 'oddfprice.b2'      '2020-09-15' '2022-01-01' '2020-01-01' '2021-01-01' 2)
$G6Cases += (PriceCase 'oddfprice.b3'      '2020-09-15' '2022-01-01' '2020-01-01' '2021-01-01' 3)
$G6Cases += (PriceCase 'oddfprice.b4'      '2020-09-15' '2022-01-01' '2020-01-01' '2021-01-01' 4)
$G6Cases += (PriceCase 'oddfprice.nq4'     '2020-06-15' '2022-01-01' '2019-01-01' '2021-01-01' 0)
$G6Cases += (PriceCase 'oddfprice.nq3'     '2020-04-10' '2022-07-01' '2019-07-01' '2021-01-01' 0)
$G6Cases += (PriceCase 'oddfprice.short'   '2020-10-01' '2022-01-01' '2020-08-01' '2021-01-01' 0)
$G6Cases += (YieldCase 'oddfyield.witness' '2020-07-01' '2022-01-01' '2020-01-01' '2021-01-01' 0)
$G6Cases += (YieldCase 'oddfyield.mid2'    '2020-09-15' '2022-01-01' '2020-01-01' '2021-01-01' 0)
$G6Cases += (YieldCase 'oddfyield.nq4'     '2020-06-15' '2022-01-01' '2019-01-01' '2021-01-01' 0)

# --- RATE : default-guess mortgage, ~586 ULP ----------------------------------
$G6Cases += @{ id='rate.mortgage'; row='RATE'; xlFn='RATE'; fsFn='Rate';
  residual='~586 ULP (0.0041666445363460975 vs 0.004166644536345589)';
  args=@(@{t='num';v=360},@{t='num';v=-1073.64},@{t='num';v=200000},@{t='num';v=0},@{t='paydue';v=0}) }

# --- IRR : solver numeric drift -----------------------------------------------
$G6Cases += @{ id='irr.unit'; row='IRR'; xlFn='IRR'; fsFn='Irr';
  residual='IRR({1,-2})=0.99999... vs 1.0 (~8000 ULP)';
  args=@(@{t='array';v=@(1,-2)}) }
$G6Cases += @{ id='irr.three'; row='IRR'; xlFn='IRR'; fsFn='Irr';
  residual='IRR({-100,50,60}) ~80 ULP';
  args=@(@{t='array';v=@(-100,50,60)}) }
$G6Cases += @{ id='irr.witness4'; row='IRR'; xlFn='IRR'; fsFn='Irr';
  residual='IRR({-10000,3000,4200,6800}) publication witness';
  args=@(@{t='array';v=@(-10000,3000,4200,6800)}) }
$G6Cases += @{ id='irr.exact121'; row='IRR'; xlFn='IRR'; fsFn='Irr';
  residual='IRR({-100,121}) exact root 0.21';
  args=@(@{t='array';v=@(-100,121)}) }
$G6Cases += @{ id='irr.mixed5'; row='IRR'; xlFn='IRR'; fsFn='Irr';
  residual='IRR 5-flow mixed-sign';
  args=@(@{t='array';v=@(-500,100,150,200,250)}) }

# --- CUMPRINC : full-schedule type 0 drift ~6 ULP -----------------------------
$G6Cases += @{ id='cumprinc.sched'; row='CUMPRINC'; xlFn='CUMPRINC'; fsFn='CumPrinc';
  residual='CUMPRINC(0.1,12,1000,1,12,0) ~6 ULP';
  args=@(@{t='num';v=0.1},@{t='num';v=12},@{t='num';v=1000},@{t='num';v=1},@{t='num';v=12},@{t='paydue';v=0}) }

# --- YIELDDISC : discounted-bill yield ~5 ULP ---------------------------------
$G6Cases += @{ id='yielddisc.witness'; row='YIELDDISC'; xlFn='YIELDDISC'; fsFn='YieldDisc';
  residual='YIELDDISC(44013,44562,95,100,0) ~5 ULP';
  args=@(@{t='date';v=44013},@{t='date';v=44562},@{t='num';v=95},@{t='num';v=100},@{t='basis';v=0}) }

# --- NPER : period-count 1 ULP ------------------------------------------------
$G6Cases += @{ id='nper.witness'; row='NPER'; xlFn='NPER'; fsFn='NPer';
  residual='~1 ULP';
  args=@(@{t='num';v=(0.08/12)},@{t='num';v=-1037.0320893591607},@{t='num';v=10000},@{t='num';v=0},@{t='paydue';v=0}) }

# --- YIELDMAT : yield-at-maturity 1 ULP ---------------------------------------
$G6Cases += @{ id='yieldmat.witness'; row='YIELDMAT'; xlFn='YIELDMAT'; fsFn='YieldMat';
  residual='~1 ULP';
  args=@(@{t='date';v=(S '2024-06-15')},@{t='date';v=(S '2025-12-31')},@{t='date';v=(S '2024-01-01')},
         @{t='num';v=0.0525},@{t='num';v=98.59811340546048},@{t='basis';v=1}) }

# --- TBILLYIELD : discounted-bill yield sub-ULP -------------------------------
$G6Cases += @{ id='tbillyield.witness'; row='TBILLYIELD'; xlFn='TBILLYIELD'; fsFn='TBillYield';
  residual='~1 ULP on some settlements';
  args=@(@{t='date';v=(S '2008-03-31')},@{t='date';v=(S '2008-06-01')},@{t='num';v=98.45}) }

$G6Cases
