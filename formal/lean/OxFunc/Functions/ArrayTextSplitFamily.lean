import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

def arrayTextSplitMetaBase : FunctionMeta := {
  functionId := "FUNC.ARRAY_TEXT_SPLIT_BASE"
  arity := { min := 1, max := 6 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def arraytotextMeta : FunctionMeta := {
  arrayTextSplitMetaBase with
  functionId := "FUNC.ARRAYTOTEXT"
  arity := { min := 1, max := 2 }
  kernelSignatureClass := KernelSignatureClass.textToText
}

def textsplitMeta : FunctionMeta := {
  arrayTextSplitMetaBase with
  functionId := "FUNC.TEXTSPLIT"
  arity := { min := 2, max := 6 }
}

def arrayToTextFormatAccepted (n : Int) : Bool :=
  n = 0 ∨ n = 1

def textSplitDefaultPad : WorksheetErrorCode :=
  WorksheetErrorCode.na

theorem arrayTextSplit_meta_profiles :
    arraytotextMeta.arity = { min := 1, max := 2 }
    ∧ textsplitMeta.arity = { min := 2, max := 6 }
    ∧ arraytotextMeta.kernelSignatureClass = KernelSignatureClass.textToText
    ∧ textsplitMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ textsplitMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [arrayTextSplitMetaBase, arraytotextMeta, textsplitMeta]

theorem arrayTextSplit_ids :
    arraytotextMeta.functionId = "FUNC.ARRAYTOTEXT"
    ∧ textsplitMeta.functionId = "FUNC.TEXTSPLIT" := by
  simp [arraytotextMeta, textsplitMeta]

theorem arrayToTextFormatAccepted_seed0 :
    arrayToTextFormatAccepted 0 = True := by
  simp [arrayToTextFormatAccepted]

theorem arrayToTextFormatAccepted_seed1 :
    arrayToTextFormatAccepted 1 = True := by
  simp [arrayToTextFormatAccepted]

theorem textSplitDefaultPad_seed :
    textSplitDefaultPad = WorksheetErrorCode.na := by
  rfl

end OxFunc.Functions
