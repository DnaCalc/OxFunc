import OxFunc.FunctionCore
import OxFunc.Functions.TextSliceFamily

namespace OxFunc.Functions

open OxFunc

def textBCompatMetaBase : FunctionMeta := {
  functionId := "FUNC.TEXT_B_COMPAT_BASE"
  arity := { min := 1, max := 1 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.none
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def findbMeta : FunctionMeta := {
  textBCompatMetaBase with
  functionId := "FUNC.FINDB"
  arity := { min := 2, max := 3 }
}

def leftbMeta : FunctionMeta := {
  textBCompatMetaBase with
  functionId := "FUNC.LEFTB"
  arity := { min := 1, max := 2 }
}

def lenbMeta : FunctionMeta := {
  textBCompatMetaBase with
  functionId := "FUNC.LENB"
}

def midbMeta : FunctionMeta := {
  textBCompatMetaBase with
  functionId := "FUNC.MIDB"
  arity := { min := 3, max := 3 }
}

def replacebMeta : FunctionMeta := {
  textBCompatMetaBase with
  functionId := "FUNC.REPLACEB"
  arity := { min := 4, max := 4 }
}

def rightbMeta : FunctionMeta := {
  textBCompatMetaBase with
  functionId := "FUNC.RIGHTB"
  arity := { min := 1, max := 2 }
}

def searchbMeta : FunctionMeta := {
  textBCompatMetaBase with
  functionId := "FUNC.SEARCHB"
  arity := { min := 2, max := 3 }
}

theorem textBCompat_meta_profiles :
    findbMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ lenbMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ replacebMeta.kernelSignatureClass = KernelSignatureClass.custom := by
  simp [textBCompatMetaBase, findbMeta, lenbMeta, replacebMeta]

theorem textBCompat_arities :
    leftbMeta.arity = { min := 1, max := 2 }
    ∧ midbMeta.arity = { min := 3, max := 3 }
    ∧ searchbMeta.arity = { min := 2, max := 3 } := by
  simp [leftbMeta, midbMeta, searchbMeta]

-- Current non-DBCS Compatibility Version 2 binding: LEFTB/RIGHTB follow the
-- character paths, while LENB/MIDB preserve raw-unit indexing.
def leftbCharactersV2 := leftCharactersV2
def rightbCharactersV2 := rightCharactersV2
def midbUtf16 := midUtf16
def lenbUtf16 := lenUtf16Units

theorem textBCompat_distinct_pair_indexing :
    leftbCharactersV2 [0xD83D,0xDE00] 1 = [0xD83D,0xDE00]
    ∧ midbUtf16 [0xD83D,0xDE00] 1 1 = [0xD83D]
    ∧ lenbUtf16 [0xD83D,0xDE00] = 2 := by
  native_decide

end OxFunc.Functions
