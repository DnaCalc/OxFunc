import OxFunc.CoercionPrimitives

namespace OxFunc

inductive RefKind where
  | a1
  | area
  | threeD
  | structured
  | spillAnchor
  deriving DecidableEq, Repr

structure ReferenceToken where
  kind : RefKind
  target : String
  deriving DecidableEq, Repr

structure ResolverCapabilities where
  allowEvalTimeDeref : Bool := true
  allowThreeDRefs : Bool := false
  allowStructuredRefs : Bool := true
  allowSpillAnchorRefs : Bool := true
  allowExternalRefs : Bool := false
  deriving DecidableEq, Repr

inductive RefResolutionError where
  | evalTimeDerefNotAllowed
  | capabilityDenied (kind : RefKind)
  | unresolved (target : String)
  deriving DecidableEq, Repr

structure ReferenceResolver where
  capabilities : ResolverCapabilities
  resolve : ReferenceToken → Except RefResolutionError CoercionInput

def isExternalTarget (target : String) : Bool :=
  target.contains '['

def capabilityAllows (caps : ResolverCapabilities) (ref : ReferenceToken) : Bool :=
  caps.allowEvalTimeDeref &&
  match ref.kind with
  | .threeD => caps.allowThreeDRefs
  | .structured => caps.allowStructuredRefs
  | .spillAnchor => caps.allowSpillAnchorRefs
  | .a1 | .area => true &&
      ((not (isExternalTarget ref.target)) || caps.allowExternalRefs)

def resolveRefToInput (resolver : ReferenceResolver) (ref : ReferenceToken) :
    Except RefResolutionError CoercionInput :=
  if resolver.capabilities.allowEvalTimeDeref then
    if capabilityAllows resolver.capabilities ref then
      resolver.resolve ref
    else
      Except.error (RefResolutionError.capabilityDenied ref.kind)
  else
    Except.error RefResolutionError.evalTimeDerefNotAllowed

def resolveRefToNumber (resolver : ReferenceResolver) (ref : ReferenceToken) :
    Except (RefResolutionError ⊕ CoercionError) Rat :=
  match resolveRefToInput resolver ref with
  | Except.error e => Except.error (Sum.inl e)
  | Except.ok v =>
      match coerceToNumber v with
      | Except.error ce => Except.error (Sum.inr ce)
      | Except.ok n => Except.ok n

def resolverDenyingThreeD : ReferenceResolver := {
  capabilities := {
    allowEvalTimeDeref := true
    allowThreeDRefs := false
    allowStructuredRefs := true
    allowSpillAnchorRefs := true
    allowExternalRefs := false
  }
  resolve := fun _ => Except.error (RefResolutionError.unresolved "not called")
}

theorem resolveRefToInput_threeD_denied :
    resolveRefToInput resolverDenyingThreeD
      { kind := .threeD, target := "Sheet1:Sheet2!A1" } =
      Except.error (RefResolutionError.capabilityDenied .threeD) := by
  simp [resolveRefToInput, capabilityAllows, resolverDenyingThreeD]

theorem resolveRefToNumber_deterministic (resolver : ReferenceResolver) (ref : ReferenceToken) :
    resolveRefToNumber resolver ref = resolveRefToNumber resolver ref := rfl

end OxFunc
