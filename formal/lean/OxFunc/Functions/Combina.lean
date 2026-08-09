import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptCombina [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def combinaMeta : FunctionMeta := {
  functionId := "FUNC.COMBINA"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.unaryNumericScalarOnly
  kernelSignatureClass := KernelSignatureClass.numsToNum
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def evalCombinaSurfaceClass (x y : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber x, coerceToNumber y with
  | .ok n, .ok k =>
      let tn := if n.num < 0 then -Int.ediv (-n.num) n.den else Int.ediv n.num n.den
      let tk := if k.num < 0 then -Int.ediv (-k.num) k.den else Int.ediv k.num k.den
      if tn = 0 ∧ tk = 0 then .ok "number"
      else if tn < 0 ∨ k < 0 then .error .num
      else
        let total := tn + tk - 1
        if total < 0 ∨ tk < 0 ∨ total < tk ∨ total > 2147483646 then .error .num
        else .ok "number"
  | .error (.worksheetError code), _ => .error code
  | _, .error (.worksheetError code) => .error code
  | .error _, _ => .error .value
  | _, .error _ => .error .value

/-!
W109 executable-route binding for the current-reference COMBINA numeric body.
COMBINA first applies the current x64 reference's DAZ treatment, then truncates
`n` and `k`. The `(0,0) → 1` pool precedes the asymmetric guard
`trunc(n) < 0 ∨ DAZ(k) < 0`. It then constructs the integer total `n+k-1` and
delegates admission and publication to the already modelled COMBIN route,
including its `2_147_483_646` maximum total. This explicitly excludes the
worksheet-visible spelling `COMBIN(n+k-1,k)`, where addition would precede
COMBIN's own argument truncation.
-/
inductive CombinaPublicationSite where
  | denormalsAreZero
  | truncateN
  | truncateK
  | zeroPoolBeforeNegativeChoiceGuard
  | asymmetricNegativeChoiceGuard
  | transformedTotalAfterTruncation
  | inheritedCombinAdmission
  | combinCyclicStoredX87Publication
  deriving DecidableEq, Repr

def combinaPublicationSchedule : List CombinaPublicationSite :=
  [.denormalsAreZero, .truncateN, .truncateK,
   .zeroPoolBeforeNegativeChoiceGuard, .asymmetricNegativeChoiceGuard,
   .transformedTotalAfterTruncation, .inheritedCombinAdmission,
   .combinCyclicStoredX87Publication]

structure CombinaPublicationRoute where
  dazBeforeTruncation : Bool
  separateArgumentTruncation : Bool
  zeroPoolBeforeNegativeChoiceGuard : Bool
  negativeGuardUsesTruncatedNAndDazChoice : Bool
  transformedTotalIsNPlusKMinusOne : Bool
  inheritsCombinMaximumTotal : Bool
  inheritsCombinComplementReduction : Bool
  inheritsCombinCyclicFactorOrder : Bool
  inheritsCombinStoredX87Publication : Bool
  deriving DecidableEq, Repr

def combinaPublicationRoute : CombinaPublicationRoute := {
  dazBeforeTruncation := true
  separateArgumentTruncation := true
  zeroPoolBeforeNegativeChoiceGuard := true
  negativeGuardUsesTruncatedNAndDazChoice := true
  transformedTotalIsNPlusKMinusOne := true
  inheritsCombinMaximumTotal := true
  inheritsCombinComplementReduction := true
  inheritsCombinCyclicFactorOrder := true
  inheritsCombinStoredX87Publication := true
}

theorem evalCombina_zero_pool_positive_choose_is_num :
    evalCombinaSurfaceClass (.number 0) (.number 1) = .error .num := by
  native_decide

theorem evalCombina_truncation_guard_and_inherited_ceiling :
    evalCombinaSurfaceClass (.number (-1 / 4)) (.number (3 / 4)) = .ok "number"
    ∧ evalCombinaSurfaceClass (.number (-1 / 4)) (.number 1) = .error .num
    ∧ evalCombinaSurfaceClass (.number 1) (.number (-1 / 4)) = .error .num
    ∧ evalCombinaSurfaceClass (.number (2147483646 + 3 / 4)) (.number 1) = .ok "number"
    ∧ evalCombinaSurfaceClass (.number (2147483647 + 1 / 4)) (.number 1) = .error .num := by
  native_decide

theorem combinaMeta_profiles :
    combinaMeta.kernelSignatureClass = KernelSignatureClass.numsToNum
    ∧ combinaMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [combinaMeta]

theorem combina_publication_route_inherits_transformed_combin_graph :
    combinaPublicationSchedule =
      [.denormalsAreZero, .truncateN, .truncateK,
       .zeroPoolBeforeNegativeChoiceGuard, .asymmetricNegativeChoiceGuard,
       .transformedTotalAfterTruncation, .inheritedCombinAdmission,
       .combinCyclicStoredX87Publication]
    ∧ combinaPublicationRoute.dazBeforeTruncation = true
    ∧ combinaPublicationRoute.separateArgumentTruncation = true
    ∧ combinaPublicationRoute.zeroPoolBeforeNegativeChoiceGuard = true
    ∧ combinaPublicationRoute.negativeGuardUsesTruncatedNAndDazChoice = true
    ∧ combinaPublicationRoute.transformedTotalIsNPlusKMinusOne = true
    ∧ combinaPublicationRoute.inheritsCombinMaximumTotal = true
    ∧ combinaPublicationRoute.inheritsCombinComplementReduction = true
    ∧ combinaPublicationRoute.inheritsCombinCyclicFactorOrder = true
    ∧ combinaPublicationRoute.inheritsCombinStoredX87Publication = true := by
  native_decide

end OxFunc.Functions
