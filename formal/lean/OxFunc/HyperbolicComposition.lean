import OxFunc.NumericPublication

namespace OxFunc.HyperbolicComposition

/-- Bind the observed arithmetic operations explicitly. Their Excel identity
is empirical; these definitions do not replace them with native primitives. -/
def sinh (exp expm1 : Float → Float) (stagedSub : Float → Float → Float)
    (x : Float) : Float :=
  if x.abs ≥ 1 then stagedSub (exp x) (exp (-x)) / 2
  else (expm1 x - expm1 (-x)) / 2

def cosh (exp : Float → Float) (stagedAdd : Float → Float → Float)
    (x : Float) : Float := stagedAdd (exp x) (exp (-x)) / 2

/-- Finite-input publication for the admitted TANH/COTH slice. -/
def saturate (input result : Float) : Float :=
  if result.isFinite then result else if input < 0 then -1 else 1

/-- The small-input denominator reuses expm1 instead of public COSH. Each
staged operation includes its binary64 store, as in the admitted Rust graph. -/
def tanhDenominator (expm1 cosh : Float → Float)
    (stagedAdd : Float → Float → Float) (x : Float) : Float :=
  if x.abs < 1 then stagedAdd (stagedAdd (expm1 x) (expm1 (-x))) 2 / 2
  else cosh x

def rawTanh (sinh expm1 cosh : Float → Float)
    (stagedAdd stagedDiv : Float → Float → Float) (x : Float) : Float :=
  stagedDiv (sinh x) (tanhDenominator expm1 cosh stagedAdd x)

def tanh (sinh expm1 cosh : Float → Float)
    (stagedAdd stagedDiv : Float → Float → Float) (x : Float) : Float :=
  saturate x (rawTanh sinh expm1 cosh stagedAdd stagedDiv x)

def csch (sinh recip : Float → Float) (x : Float) : Except WorksheetErrorCode Float :=
  let s := sinh x
  if s == 0 then .error .div0 else .ok (NumericPublication.flushTiny (recip s))

def coth (rawTanh recip : Float → Float) (x : Float) : Except WorksheetErrorCode Float :=
  let t := rawTanh x
  if t == 0 then .error .div0 else .ok (saturate x (recip t))

theorem positive_overflow_saturates :
    (saturate 710 (Float.ofBits 0x7ff0000000000000)).toBits = (1 : Float).toBits := by native_decide
theorem negative_overflow_saturates :
    (saturate (-710) (Float.ofBits 0xfff8000000000000)).toBits = (-1 : Float).toBits := by native_decide
theorem csch_tiny_publication :
    (csch (fun _ => 1) (fun _ => Float.ofBits 1) 710).toOption.map Float.toBits = some 0 := by native_decide

end OxFunc.HyperbolicComposition
