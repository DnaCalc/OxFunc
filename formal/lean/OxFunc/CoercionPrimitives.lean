import OxFunc.ValueUniverse

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

def parseSimpleNumber : String → Option Rat
  | "0" => some 0
  | "1" => some 1
  | "-1" => some (-1)
  | _ => none

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
  simp [coerceToNumber, parseSimpleNumber]

end OxFunc
