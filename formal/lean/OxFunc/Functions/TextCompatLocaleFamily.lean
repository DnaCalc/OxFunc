import OxFunc.FunctionCore
import OxFunc.HostInfoSeam

namespace OxFunc.Functions

open OxFunc

def textCompatLocaleBase : FunctionMeta := {
  functionId := "FUNC.TEXT_COMPAT_LOCALE_BASE"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.applicationState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.none
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.composite
  surfaceFecDependencyProfile := FecDependencyProfile.composite
}

def ascMeta : FunctionMeta := { textCompatLocaleBase with functionId := "FUNC.ASC" }
def dbcsMeta : FunctionMeta := { textCompatLocaleBase with functionId := "FUNC.DBCS" }
def jisMeta : FunctionMeta := { textCompatLocaleBase with functionId := "FUNC.JIS" }

def widthConversionModeForCurrentHost (fn : WidthConversionFunction) : WidthConversionMode :=
  match fn with
  | .asc => .passThrough
  | .dbcs => .passThrough
  | .jis => .unavailable

theorem textCompatLocale_profiles :
    ascMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ dbcsMeta.threadSafety = ThreadSafetyClass.hostSerialized
    ∧ jisMeta.surfaceFecDependencyProfile = FecDependencyProfile.composite := by
  simp [textCompatLocaleBase, ascMeta, dbcsMeta, jisMeta]

theorem currentHostWidthConversionBaseline :
    widthConversionModeForCurrentHost .asc = WidthConversionMode.passThrough
    ∧ widthConversionModeForCurrentHost .dbcs = WidthConversionMode.passThrough
    ∧ widthConversionModeForCurrentHost .jis = WidthConversionMode.unavailable := by
  simp [widthConversionModeForCurrentHost]

end OxFunc.Functions
