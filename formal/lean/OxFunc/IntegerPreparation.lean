import OxFunc.NumericTextRendering

namespace OxFunc.IntegerPreparation

/-- Exact finite binary64 magnitude decoding for the executable integer model.
Nonfinite bit patterns are excluded by the caller's admission branch. -/
def binary64Magnitude (x : Float) : Rat :=
  let bits : Nat := x.toBits.toNat
  let exponent : Nat := (bits / 2 ^ 52) % 2048
  let mantissa : Nat := bits % 2 ^ 52 + if exponent = 0 then 0 else 2 ^ 52
  let shift : Int := (if exponent = 0 then -1022 else (exponent : Int) - 1023) - 52
  if shift < 0 then (mantissa : Rat) / (2 ^ shift.natAbs : Nat)
  else (mantissa * 2 ^ shift.toNat : Nat)

/-- Mirrors format .30e (31 significant digits, nearest-even), then initial
15 digits with ties away. This explicitly records the runtime decimal layer;
the generic formatter's different tie rule is not reused as an INT claim. -/
def initial15Away (magnitude : Rat) (exponent : Int) : NumericTextDecimal :=
  let shift := 30 - exponent
  let scaled := if shift ≥ 0 then magnitude * (10 ^ shift.toNat : Nat)
    else magnitude / (10 ^ shift.natAbs : Nat)
  let head := scaled.num.natAbs / scaled.den
  let rest := scaled.num.natAbs % scaled.den
  let digits := head + if 2 * rest > scaled.den ||
    (2 * rest = scaled.den && head % 2 = 1) then 1 else 0
  let (digits, exponent) := if digits ≥ 10 ^ 31 then (digits / 10, exponent + 1)
    else (digits, exponent)
  let divisor := 10 ^ 16
  ⟨digits / divisor + (if 2 * (digits % divisor) ≥ divisor then 1 else 0), exponent - 14⟩

def preparedMagnitude (x : Float) : Rat :=
  let magnitude := binary64Magnitude x
  let exponent := numericTextExponent 650 magnitude 0
  let pair := initial15Away magnitude exponent
  if pair.scale < 0 then (pair.digits : Rat) / (10 ^ pair.scale.natAbs : Nat)
  else (pair.digits * 10 ^ pair.scale.toNat : Nat)

def intKernel (x : Float) : Float :=
  if !x.isFinite || x == x.floor then x
  else if x.abs ≥ 1.0e15 then
    if x < 0 then -(x.abs + 1).floor else x.floor
  else
    let magnitude := preparedMagnitude x
    let value := if x < 0 then -magnitude else magnitude
    let integer := value.floor
    let magnitude := Float.ofScientific integer.natAbs false 0
    if integer < 0 then -magnitude else magnitude

def isEvenKernel (x : Float) : Bool :=
  let value := (x.abs + 1.0e-10).floor
  if value ≥ 9007199254740992 then true else value.toUInt64 % 2 == 0

def isOddKernel (x : Float) : Bool := !isEvenKernel x

theorem int_positive_decimal_midpoint :
    (intKernel 962306618417396.5).toBits = (962306618417397 : Float).toBits := by
  native_decide
theorem int_negative_decimal_midpoint :
    (intKernel (-962306618417396.5)).toBits = (-962306618417397 : Float).toBits := by
  native_decide
theorem int_preserves_large_integer :
    (intKernel 10000000000000002).toBits = (10000000000000002 : Float).toBits := by
  native_decide

theorem int_negative_power_predecessor_increment :
    (intKernel (Float.ofBits 0xc30fffffffffffff)).toBits =
      (-1125899906842625 : Float).toBits := by native_decide
theorem int_neighbor_below_one :
    (intKernel (Float.ofBits 0x3fefffffffffffff)).toBits = (1 : Float).toBits := by
  native_decide
theorem parity_addition_boundary :
    isEvenKernel 0.9999999999 = false ∧ isOddKernel (-0.9999999999) = true := by
  native_decide
theorem parity_large_finite_integer : isEvenKernel 1.0e100 = true := by
  native_decide

end OxFunc.IntegerPreparation
