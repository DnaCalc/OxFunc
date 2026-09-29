import OxFunc.ValueUniverse

namespace OxFunc

-- W111 generic Number-to-Text / ADDRESS formatting substrate. The caller
-- supplies the decimal exponent of the finite nonzero binary64 magnitude.
-- The runtime currently rounds through a 31-significant-decimal intermediate.
-- The exact-15 alternative is retained as an explicit research hypothesis:
-- precision3204 disproves universal Excel identity for either initial rule.
structure NumericTextDecimal where
  digits : Nat
  scale : Int
  deriving Repr, DecidableEq

def numericTextExactInitialHypothesis (magnitude : Rat) (exponent : Int) : NumericTextDecimal :=
  let shift := 14 - exponent
  let scaled := if shift ≥ 0 then magnitude * (10 ^ shift.toNat : Nat)
    else magnitude / (10 ^ (-shift).toNat : Nat)
  let numerator := scaled.num.natAbs
  let head := numerator / scaled.den
  let rest := numerator % scaled.den
  ⟨head + (if 2 * rest > scaled.den then 1 else 0), exponent - 14⟩

def numericTextRuntimeInitialDecimal (magnitude : Rat) (exponent : Int) : NumericTextDecimal :=
  let shift := 30 - exponent
  let scaled := if shift ≥ 0 then magnitude * (10 ^ shift.toNat : Nat)
    else magnitude / (10 ^ (-shift).toNat : Nat)
  let head := scaled.num.natAbs / scaled.den
  let rest := scaled.num.natAbs % scaled.den
  let digits := head + (if 2 * rest > scaled.den ||
      (2 * rest = scaled.den && head % 2 = 1) then 1 else 0)
  let (digits, exponent) := if digits ≥ 10 ^ 31 then (digits / 10, exponent + 1)
    else (digits, exponent)
  let divisor := 10 ^ 16
  ⟨digits / divisor + (if 2 * (digits % divisor) > divisor then 1 else 0), exponent - 14⟩

def numericTextNormalize : Nat → NumericTextDecimal → NumericTextDecimal
  | 0, pair => pair
  | fuel + 1, pair =>
      if pair.digits != 0 && pair.digits % 10 = 0 then
        numericTextNormalize fuel ⟨pair.digits / 10, pair.scale + 1⟩
      else pair

def numericTextReduceDigits (digits divisor : Nat) : Nat :=
  digits / divisor + if 2 * (digits % divisor) ≥ divisor then 1 else 0

def numericTextScientificDigits (exponent : Int) : Nat :=
  if exponent.natAbs ≥ 99 then 14 else 15

def numericTextFixed (pair : NumericTextDecimal) : String :=
  let chars := (toString pair.digits).toList
  let point := (chars.length : Int) + pair.scale
  if point ≤ 0 then "0." ++ String.ofList (List.replicate (-point).toNat '0' ++ chars)
  else if point ≥ chars.length then
    String.ofList (chars ++ List.replicate (point - chars.length).toNat '0')
  else String.ofList (chars.take point.toNat ++ ['.'] ++ chars.drop point.toNat)

def numericTextScientific (pair : NumericTextDecimal) : String :=
  let exponent := ((toString pair.digits).length : Int) + pair.scale - 1
  let excess := (toString pair.digits).length - numericTextScientificDigits exponent
  let pair := if excess = 0 then pair else
    numericTextNormalize 20
      ⟨numericTextReduceDigits pair.digits (10 ^ excess), pair.scale + excess⟩
  let chars := (toString pair.digits).toList
  let exponent := (chars.length : Int) + pair.scale - 1
  let mantissa := if chars.length = 1 then String.ofList chars
    else String.ofList (chars.take 1 ++ ['.'] ++ chars.drop 1)
  let exponentText := toString exponent.natAbs
  mantissa ++ "E" ++ (if exponent < 0 then "-" else "+") ++
    (if exponentText.length < 2 then "0" else "") ++ exponentText

def numericTextRender (negative : Bool) (input : NumericTextDecimal) : String :=
  if input.digits = 0 then "0" else
    let pair := numericTextNormalize 20 input
    let fixed := numericTextFixed pair
    let unsigned := if fixed.length ≤ 20 then fixed else numericTextScientific pair
    (if negative then "-" else "") ++ unsigned

def numericTextExponent : Nat → Rat → Int → Int
  | 0, _, exponent => exponent
  | fuel + 1, magnitude, exponent =>
      if magnitude < 1 then numericTextExponent fuel (magnitude * 10) (exponent - 1)
      else if magnitude ≥ 10 then numericTextExponent fuel (magnitude / 10) (exponent + 1)
      else exponent

-- Bounded exponent search covers all finite binary64 normal values; this
-- model does not claim the direct nonfinite/subnormal carrier fallback.
def numericTextRuntimeForRat (value : Rat) : String :=
  if value = 0 then "0" else
    let magnitude := if value < 0 then -value else value
    let exponent := numericTextExponent 650 magnitude 0
    numericTextRender (value < 0) (numericTextRuntimeInitialDecimal magnitude exponent)

theorem numericText_initial_ties_toward_zero :
    numericTextRuntimeInitialDecimal (200000000000001 / 2) 14 = ⟨100000000000000, 0⟩
    ∧ numericTextRuntimeInitialDecimal (246913578024689 / 2) 14 = ⟨123456789012344, 0⟩ := by
  native_decide

theorem numericText_runtime_precision_differs_from_exact_hypothesis :
    let x : Rat := (1000000000000005 * 10 ^ 17 + 1) / 10 ^ 18
    numericTextRuntimeInitialDecimal x 14 = ⟨100000000000000, 0⟩
    ∧ numericTextExactInitialHypothesis x 14 = ⟨100000000000001, 0⟩ := by
  native_decide

theorem numericText_unsigned_fixed_width :
    numericTextRender false ⟨123456789012345, 5⟩ = "12345678901234500000"
    ∧ numericTextRender true ⟨123456789012345, 5⟩ = "-12345678901234500000"
    ∧ numericTextRender false ⟨123456789012345, 6⟩ = "1.23456789012345E+20" := by
  native_decide

theorem numericText_scientific_second_round :
    numericTextRender false ⟨123456789012345, 84⟩ = "1.23456789012345E+98"
    ∧ numericTextRender false ⟨123456789012345, 85⟩ = "1.2345678901235E+99"
    ∧ numericTextRender false ⟨999999999999995, 85⟩ = "1E+100"
    ∧ numericTextRender false ⟨179769313486232, 294⟩ = "1.7976931348623E+308" := by
  native_decide

end OxFunc
