import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def lookupProbFrequencyBaseMeta : FunctionMeta := {
  functionId := "FUNC.LOOKUP_PROB_FREQUENCY_BASE"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def lookupMeta : FunctionMeta := { lookupProbFrequencyBaseMeta with functionId := "FUNC.LOOKUP", arity := { min := 2, max := 3 } }
def frequencyMeta : FunctionMeta := { lookupProbFrequencyBaseMeta with functionId := "FUNC.FREQUENCY", arity := Arity.exact 2 }
def probMeta : FunctionMeta := { lookupProbFrequencyBaseMeta with functionId := "FUNC.PROB", arity := { min := 3, max := 4 } }
def modeMultMeta : FunctionMeta := { lookupProbFrequencyBaseMeta with functionId := "FUNC.MODE.MULT", arity := { min := 1, max := 255 } }

/--
Batch 77 keeps a bounded lookup/statistical slice only: 1-D approximate numeric LOOKUP,
vertical FREQUENCY publication, strict discrete PROB vectors, and vertical MODE.MULT output.
Shared dispatch/export wiring and broader array publication remain intentionally out of scope.
-/
theorem lookupProbFrequencyFamily_meta_profiles :
    lookupMeta.arity = { min := 2, max := 3 }
    ∧ frequencyMeta.arity = Arity.exact 2
    ∧ probMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ modeMultMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ probMeta.threadSafety = ThreadSafetyClass.safePure := by
  simp [lookupProbFrequencyBaseMeta, lookupMeta, frequencyMeta, probMeta, modeMultMeta]

theorem lookupProbFrequencyFamily_ids :
    lookupMeta.functionId = "FUNC.LOOKUP"
    ∧ frequencyMeta.functionId = "FUNC.FREQUENCY"
    ∧ probMeta.functionId = "FUNC.PROB"
    ∧ modeMultMeta.functionId = "FUNC.MODE.MULT" := by
  simp [lookupMeta, frequencyMeta, probMeta, modeMultMeta]

end OxFunc.Functions
