import OxFunc.NumericPublication

namespace OxFunc.ElementaryPublication

/-- Empirical stored-primitive binding; no universal logarithm identity proof. -/
def log (ln : Float → Float) (divide : Float → Float → Float)
    (number base : Float) : Except WorksheetErrorCode Float :=
  if number ≤ 0 then .error .num
  else if base ≤ 0 then .error .num
  else if base == 1 then .error .div0
  else
    let value := divide (ln number) (ln base)
    .ok (if value == 0 then 0 else value)

def fisher (ln : Float → Float)
    (add subtract divide : Float → Float → Float) (x : Float) :
    Except WorksheetErrorCode Float :=
  if x.abs ≥ 1 then .error .num
  else .ok (ln (divide (add 1 x) (subtract 1 x)) / 2)

/-- Admitted even-median numeric slice, with lower ≤ upper after sorting. -/
def medianEven (lower upper : Float) : Except WorksheetErrorCode Float :=
  let halfDifference := (upper - lower) / 2
  let result := lower + NumericPublication.flushTiny halfDifference
  if result.isFinite then .ok result else .error .num

def repeatSquare (multiply : Float → Float → Float) :
    Nat → Float → Nat → Float → Float
  | 0, _, _, accumulator => accumulator
  | fuel + 1, base, exponent, accumulator =>
    if exponent == 0 then accumulator else
      let next := if exponent % 2 == 1 then multiply accumulator base else accumulator
      if exponent / 2 == 0 then next
      else repeatSquare multiply fuel (multiply base base) (exponent / 2) next

/-- The unsigned32 sentinel is excluded from the integer publication path. -/
def integerMagnitudeAdmitted (magnitude : Nat) : Bool := magnitude < 4294967295

/-- The bounded magnitude's effective decimal scale at positive base ten. -/
def integerDecimalScale (magnitude : Nat) : Int :=
  if magnitude < 2147483648 then Int.ofNat magnitude
  else Int.ofNat magnitude - 4294967296

/-- Nonzero base and positive integer magnitude admitted below unsigned32's
sentinel. The caller delegates other integers to its noninteger body.
Negative fractional-root admission remains outside this binding's slice. -/
def integerPower (multiply : Float → Float → Float)
    (reciprocal : Float → Float) (decimalPower : Int → Float)
    (base : Float) (magnitude : Nat) (negative : Bool) :
    Except WorksheetErrorCode Float :=
  let positive := if base == 10 then
      let scale := integerDecimalScale magnitude
      if scale > 308 then Float.ofBits 0x7ff0000000000000
      else if scale < -308 then 0 else decimalPower scale
    else repeatSquare multiply 32 base magnitude 1
  if negative then
    if positive.abs < Float.ofBits 0x0010000000000000 then .error .div0
    else
      let result := reciprocal positive
      if result.isFinite then .ok (NumericPublication.flushTiny result)
      else .error .num
  else if positive.isFinite then .ok (NumericPublication.flushTiny positive)
  else .error .num

theorem log_zero_is_positive :
    ((log (fun x => if x == 1 then 0 else -1) (fun x y => x / y) 1 0.5).toOption.map Float.toBits) = some 0 := by
  native_decide

theorem median_half_difference_flush :
    ((medianEven (Float.ofBits 0x0010000000000000)
      (Float.ofBits 0x0010000000000002)).toOption.map Float.toBits) =
      some 0x0010000000000000 := by native_decide

theorem integer_reciprocal_tiny_is_div0 :
    (integerPower (fun _ _ => Float.ofBits 1) (fun x => 1 / x)
      (fun _ => 1) 2 2 true).toOption = none := by native_decide

theorem integer_width_sentinel_and_decimal_wrap :
    integerMagnitudeAdmitted 4294967294 = true ∧
    integerMagnitudeAdmitted 4294967295 = false ∧
    integerDecimalScale 2147483647 = 2147483647 ∧
    integerDecimalScale 2147483648 = -2147483648 ∧
    integerDecimalScale 4294967294 = -2 := by native_decide

end OxFunc.ElementaryPublication
