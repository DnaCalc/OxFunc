import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def cosMeta : FunctionMeta := {
  functionId := "FUNC.COS"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
  kernelSignatureClass := KernelSignatureClass.numToNum
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def evalCosSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

/--
W109 executable publication-route tag. This records the semantically
load-bearing branch structure without duplicating the x87 numeric backend in
Lean: the tiny exact-one guard precedes the reduced-quadrant dispatch, even
quadrants use FCOS, and odd quadrants use the tangent-square reconstruction.
-/
inductive CosPublicationRoute where
  | tinyExactOne
  | evenQuadrantFcos
  | oddQuadrantTangentSquare
  deriving DecidableEq, Repr

def cosPublicationRoute (belowTinyGuard evenQuadrant : Bool) : CosPublicationRoute :=
  if belowTinyGuard then
    .tinyExactOne
  else if evenQuadrant then
    .evenQuadrantFcos
  else
    .oddQuadrantTangentSquare

theorem evalCos_numeric_text_admitted :
    evalCosSurfaceClass (.text "1") = .ok "number" := by
  simp [evalCosSurfaceClass, coerceToNumber, parseSimpleNumber]

theorem cosMeta_profiles :
    cosMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ cosMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [cosMeta]

theorem cos_publication_route_order_and_split :
    cosPublicationRoute true false = .tinyExactOne
    ∧ cosPublicationRoute false true = .evenQuadrantFcos
    ∧ cosPublicationRoute false false = .oddQuadrantTangentSquare := by
  native_decide

end OxFunc.Functions
