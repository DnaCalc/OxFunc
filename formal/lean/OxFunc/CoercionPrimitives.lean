import OxFunc.ValueUniverse
import OxFunc.DecimalTextConversion

namespace OxFunc

inductive CoercionError where
  | missingArg
  | emptyCell
  | nonNumericText (s : String)
  | worksheetError (code : WorksheetErrorCode)
  | unsupportedKind (kind : String)
  deriving DecidableEq, Repr

inductive CoercionInput where
  | number (n : Rat)
  | text (s : String)
  | logical (b : Bool)
  | error (code : WorksheetErrorCode)
  | missingArg
  | emptyCell
  deriving DecidableEq, Repr

-- W111 numeric-text decoration substrate. Locale-sensitive grouping,
-- currency, native digits and dates remain an explicit preparation-context
-- dependency; this model covers the locale-independent ASCII decimal lane.
def trimNumericSpaces (chars : List Char) : List Char :=
  (chars.dropWhile (· == ' ')).reverse.dropWhile (· == ' ') |>.reverse

structure NumericTextDecorations where
  body : List Char
  negative : Bool := false
  percent : Bool := false
  deriving DecidableEq, Repr

def numericTextDecorations : Nat → List Char → Bool → Bool → Bool → Option NumericTextDecorations
  | 0, _, _, _, _ => none
  | fuel + 1, chars, signed, negative, percent =>
      let chars := trimNumericSpaces chars
      if chars.head? = some '%' then
        if percent then none else numericTextDecorations fuel chars.tail signed negative true
      else if chars.getLast? = some '%' then
        if percent then none else numericTextDecorations fuel chars.dropLast signed negative true
      else if chars.head? = some '+' || chars.head? = some '-' then
        if signed then none else
          numericTextDecorations fuel chars.tail true (chars.head? = some '-') percent
      else if chars.head? = some '(' && chars.getLast? = some ')' then
        if signed then none else numericTextDecorations fuel chars.tail.dropLast true true percent
      else if chars.isEmpty then none else some ⟨chars, negative, percent⟩

def asciiDigitsValue (chars : List Char) : Option Nat :=
  if chars.isEmpty || !chars.all (fun c => decide ('0' ≤ c ∧ c ≤ '9')) then none
  else some (chars.foldl (fun n c => 10 * n + c.toNat - 48) 0)

def asciiSignedInteger (chars : List Char) : Option Int := do
  let (negative, digits) := match chars with
    | '-' :: rest => (true, rest)
    | '+' :: rest => (false, rest)
    | _ => (false, chars)
  let magnitude ← asciiDigitsValue digits
  pure (if negative then -(Int.ofNat magnitude) else Int.ofNat magnitude)

def asciiDecimalMagnitude (text : String) : Option Rat := do
  let (whole, fraction) ← match text.splitOn "." with
    | [whole] => some (whole, "")
    | [whole, fraction] => some (whole, fraction)
    | _ => none
  let digits := whole.toList ++ fraction.toList
  let _ ← asciiDigitsValue digits
  let significant := digits.dropWhile (· == '0')
  if significant.isEmpty then pure 0 else
    let kept ← asciiDigitsValue (significant.take 15)
    let truncated := kept * 10 ^ (significant.length - 15)
    pure ((Rat.ofInt (Int.ofNat truncated)) / (Rat.ofInt (Int.ofNat (10 ^ fraction.length))))

def asciiDecimalNumber (chars : List Char) (scaleAdjustment : Int := 0) : Option Rat := do
  let (negative, chars) := match chars with
    | '-' :: rest => (true, rest)
    | '+' :: rest => (false, rest)
    | _ => (false, chars)
  let parts := (String.ofList chars).toLower.splitOn "e"
  let (mantissa, exponent) ← match parts with
    | [body] => some (body, (0 : Int))
    | [body, exponent] => do pure (body, ← asciiSignedInteger exponent.toList)
    | _ => none
  let magnitude ← asciiDecimalMagnitude mantissa
  let fractionLength := match mantissa.splitOn "." with
    | [_, fraction] => fraction.length
    | _ => 0
  let significantLength := (mantissa.toList.filter (· != '.')).dropWhile (· == '0') |>.length
  let exponent := exponent + scaleAdjustment
  let position := exponent - Int.ofNat fractionLength + Int.ofNat significantLength
  if position < -308 || position > 308 then none else do
    let scaled := if exponent < 0 then magnitude / Rat.ofInt (Int.ofNat (10 ^ exponent.natAbs))
      else magnitude * Rat.ofInt (Int.ofNat (10 ^ exponent.natAbs))
    pure (if negative then -scaled else scaled)

def parseSimpleNumber (text : String) : Option Rat := do
  let decorated ← numericTextDecorations 4 text.toList false false false
  let number ← asciiDecimalNumber decorated.body (if decorated.percent then -2 else 0)
  let number := if decorated.negative then -number else number
  pure number

-- This grammar model retains exact rational values after lexical truncation.
-- The executable DecimalTextConversion substrate binds the retained integer
-- significand and adjusted decimal scale to staged binary rounding/publication.
theorem numericTextDecoration_observed_forms :
    parseSimpleNumber " 2 " = some 2
    ∧ parseSimpleNumber "% 200" = some 2
    ∧ parseSimpleNumber "(200%)" = some (-2)
    ∧ parseSimpleNumber "(2) %" = some (-1 / 50)
    ∧ parseSimpleNumber "2e1" = some 20
    ∧ parseSimpleNumber "-% 2" = some (-1 / 50)
    ∧ parseSimpleNumber "1.234567890123456789" = some (123456789012345 / 100000000000000)
    ∧ parseSimpleNumber "(-2)" = none
    ∧ parseSimpleNumber "2%%" = none
    ∧ parseSimpleNumber "0e309" = none
    ∧ parseSimpleNumber "0.0e309" = some 0
    ∧ parseSimpleNumber "1e310%" = none
    ∧ parseSimpleNumber "0e-308%" = none
    ∧ parseSimpleNumber "2\n" = none := by
  native_decide

def coerceToNumber : CoercionInput → Except CoercionError Rat
  | .number n => Except.ok n
  | .logical true => Except.ok 1
  | .logical false => Except.ok 0
  | .text s =>
      match parseSimpleNumber s with
      | some n => Except.ok n
      | none => Except.error (CoercionError.nonNumericText s)
  | .error code => Except.error (CoercionError.worksheetError code)
  | .missingArg => Except.error CoercionError.missingArg
  | .emptyCell => Except.error CoercionError.emptyCell

theorem coerceToNumber_missingArg :
    coerceToNumber .missingArg = Except.error CoercionError.missingArg := by
  simp [coerceToNumber]

theorem coerceToNumber_emptyCell :
    coerceToNumber .emptyCell = Except.error CoercionError.emptyCell := by
  simp [coerceToNumber]

theorem coerceToNumber_logical_true :
    coerceToNumber (.logical true) = Except.ok 1 := by
  simp [coerceToNumber]

theorem coerceToNumber_text_bad :
    coerceToNumber (.text "asd") = Except.error (CoercionError.nonNumericText "asd") := by
  have h : parseSimpleNumber "asd" = none := by native_decide
  simp [coerceToNumber, h]

end OxFunc
