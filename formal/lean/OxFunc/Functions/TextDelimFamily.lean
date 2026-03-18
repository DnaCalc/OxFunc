import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

def textDelimMetaBase : FunctionMeta := {
  functionId := "FUNC.TEXT_DELIM_BASE"
  arity := { min := 2, max := 6 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.textToText
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def textafterMeta : FunctionMeta := {
  textDelimMetaBase with
  functionId := "FUNC.TEXTAFTER"
}

def textbeforeMeta : FunctionMeta := {
  textDelimMetaBase with
  functionId := "FUNC.TEXTBEFORE"
}

def textDelimAsciiText (s : String) : ExcelText :=
  { utf16CodeUnits := s.toList.map (fun c => c.val.toUInt16) }

def textAfterEmptyDelimiter (input : ExcelText) (instanceNegative : Bool) : ExcelText :=
  if instanceNegative then textDelimAsciiText "" else input

def textBeforeEmptyDelimiter (input : ExcelText) (instanceNegative : Bool) : ExcelText :=
  if instanceNegative then input else textDelimAsciiText ""

theorem textDelim_meta_profiles :
    textafterMeta.arity = { min := 2, max := 6 }
    ∧ textbeforeMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ textafterMeta.kernelSignatureClass = KernelSignatureClass.textToText
    ∧ textbeforeMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [textDelimMetaBase, textafterMeta, textbeforeMeta]

theorem textDelim_ids :
    textafterMeta.functionId = "FUNC.TEXTAFTER"
    ∧ textbeforeMeta.functionId = "FUNC.TEXTBEFORE" := by
  simp [textafterMeta, textbeforeMeta]

theorem textDelim_empty_delimiter_after_positive_seed :
    textAfterEmptyDelimiter (textDelimAsciiText "abc") false = textDelimAsciiText "abc" := by
  rfl

theorem textDelim_empty_delimiter_before_negative_seed :
    textBeforeEmptyDelimiter (textDelimAsciiText "abc") true = textDelimAsciiText "abc" := by
  rfl

end OxFunc.Functions
