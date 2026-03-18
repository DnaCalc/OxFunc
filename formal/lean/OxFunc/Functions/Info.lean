import OxFunc.FunctionCore
import OxFunc.HostInfoSeam

namespace OxFunc.Functions

open OxFunc

def infoMeta : FunctionMeta := {
  functionId := "FUNC.INFO"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.volatileContextual
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.composite
  surfaceFecDependencyProfile := FecDependencyProfile.composite
}

def normalizeInfoTypeText (s : String) : Option InfoQuery :=
  match s.trimAscii.toString.toLower with
  | "directory" => some .directory
  | "numfile" => some .numFile
  | "origin" => some .origin
  | "osversion" => some .osVersion
  | "recalc" => some .recalc
  | "release" => some .release
  | "system" => some .system
  | "memavail" => some .memAvail
  | "memused" => some .memUsed
  | "totmem" => some .totMem
  | _ => none

theorem normalizeInfoTypeText_release :
    normalizeInfoTypeText " release " = some .release := by
  native_decide

theorem infoMeta_profiles :
    infoMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ infoMeta.fecDependencyProfile = FecDependencyProfile.composite
    ∧ infoMeta.surfaceFecDependencyProfile = FecDependencyProfile.composite := by
  simp [infoMeta]

end OxFunc.Functions
