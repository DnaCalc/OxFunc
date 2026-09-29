import OxFunc.FunctionCore
import OxFunc.Functions.DistributionArguments
import OxFunc.NumericPublication

namespace OxFunc.Functions

open OxFunc

def discreteDistBaseMeta : FunctionMeta := {
  functionId := "FUNC.DISCRETE_DIST_BASE"
  arity := Arity.exact 1
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

def binomDistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.BINOM.DIST"
  arity := Arity.exact 4
}

def binomDistRangeMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.BINOM.DIST.RANGE"
  arity := { min := 3, max := 4 }
}

def binomInvMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.BINOM.INV"
  arity := Arity.exact 3
}

def binomdistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.BINOMDIST"
  arity := Arity.exact 4
}

def critbinomMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.CRITBINOM"
  arity := Arity.exact 3
}

def poissonMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.POISSON"
  arity := Arity.exact 3
}

def poissonDistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.POISSON.DIST"
  arity := Arity.exact 3
}

def hypgeomDistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.HYPGEOM.DIST"
  arity := Arity.exact 5
}

def hypgeomdistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.HYPGEOMDIST"
  arity := Arity.exact 4
}

def negbinomDistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.NEGBINOM.DIST"
  arity := Arity.exact 4
}

def negbinomdistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.NEGBINOMDIST"
  arity := Arity.exact 3
}

def exponDistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.EXPON.DIST"
  arity := Arity.exact 3
}

def expondistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.EXPONDIST"
  arity := Arity.exact 3
}

theorem discreteDistFamily_profiles :
    binomDistMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ binomDistRangeMeta.arity = { min := 3, max := 4 }
    ∧ binomInvMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ poissonMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ poissonDistMeta.arity = Arity.exact 3
    ∧ hypgeomDistMeta.arity = Arity.exact 5
    ∧ hypgeomdistMeta.arity = Arity.exact 4
    ∧ negbinomDistMeta.arity = Arity.exact 4
    ∧ negbinomdistMeta.arity = Arity.exact 3
    ∧ exponDistMeta.arity = Arity.exact 3
    ∧ expondistMeta.arity = Arity.exact 3 := by
  simp [
    discreteDistBaseMeta, binomDistMeta, binomDistRangeMeta, binomInvMeta, poissonMeta, poissonDistMeta, hypgeomDistMeta, hypgeomdistMeta,
    negbinomDistMeta, negbinomdistMeta, exponDistMeta, expondistMeta
  ]

def exponDistPreparedArguments := distributionPrepared 2

def binomDistPreparedArguments := distributionPrepared 3
def binomInvPreparedArguments := distributionNumericPrepared 3
def poissonPreparedArguments := distributionPrepared 2
def hypgeomDistPreparedArguments := distributionPrepared 4
def hypgeomdistPreparedArguments := distributionNumericPrepared 4
def negbinomDistPreparedArguments := distributionPrepared 3
def negbinomdistPreparedArguments := distributionNumericPrepared 3

def binomDistRangePreparedArguments (args : List CoercionInput) :
    Except WorksheetErrorCode (List Rat) :=
  if args.length == 3 || args.length == 4 then
    let effective := if args.length == 4 && args[3]? == some .missingArg then args.take 3 else args
    effective.mapM distributionNumber
  else .error .value

theorem binomial_optional_upper_missing_uses_default :
    binomDistRangePreparedArguments [.number 2, .number (1/2), .number 1] = .ok [2, 1/2, 1] ∧
    binomDistRangePreparedArguments [.number 2, .number (1/2), .number 1, .missingArg] = .ok [2, 1/2, 1] := by
  exact ⟨rfl, rfl⟩

theorem poisson_cumulative_numeric_text_rejected :
    poissonPreparedArguments [.number 0, .number 1, .text "1"] = .error .value := by rfl

/-- Finite input branch order; `expPublished` carries the observed exponential
result, including subnormals and overflow errors. `none` delegates to the
existing positive-count backend. The caller supplies integer conversion. -/
def poissonFiniteBranch (expPublished : Rat → Except WorksheetErrorCode Rat)
    (rawCount mean : Rat) (count : Int) : Except WorksheetErrorCode (Option Rat) :=
  if rawCount < 0 then .error .num
  else if count == 0 then (expPublished (-mean)).map some
  else if mean < 0 then .error .num
  else .ok none

theorem poisson_negative_raw_count_precedes_zero_branch :
    poissonFiniteBranch (fun _ => .ok 7) (-(1/10)) (-1) 0 = .error .num := by
  have negative : (-(1/10) : Rat) < 0 := by native_decide
  simp [poissonFiniteBranch, negative]

theorem poisson_zero_count_precedes_negative_mean :
    poissonFiniteBranch (fun _ => .ok 7) (1/10) (-1) 0 = .ok (some 7) := by
  have nonnegative : ¬ ((1/10 : Rat) < 0) := by native_decide
  simp [poissonFiniteBranch, nonnegative, Except.map]

theorem poisson_positive_count_rejects_negative_mean :
    poissonFiniteBranch (fun _ => .ok 7) 1 (-1) 1 = .error .num := by rfl

/-- W111 positive-count publication. `minNormal` is the binary64 minimum-normal
boundary in the Rust binding. The zero-count exponential bypasses this step. -/
def poissonPositiveCountPublication (minNormal value : Rat) : Rat :=
  let magnitude := if value < 0 then -value else value
  if magnitude < minNormal then 0 else value

/-- Finite prepared dataflow with supplied exponential and positive-count
backend. This binds branch/publication order without asserting that the
currently partial positive-count arithmetic agrees with Excel. -/
def poissonFiniteWithPublication (minNormal : Rat)
    (expPublished : Rat → Except WorksheetErrorCode Rat)
    (positiveBackend : Int → Rat → Except WorksheetErrorCode Rat)
    (rawCount mean : Rat) (count : Int) : Except WorksheetErrorCode Rat := do
  let branch ← poissonFiniteBranch expPublished rawCount mean count
  match branch with
  | some value => .ok value
  | none => return poissonPositiveCountPublication minNormal (← positiveBackend count mean)

theorem poisson_zero_count_bypasses_probability_publication :
    poissonFiniteWithPublication 1 (fun _ => .ok (1/2)) (fun _ _ => .ok 7)
      0 2 0 = .ok (1/2) := by rfl

theorem poisson_positive_count_probability_publication :
    poissonFiniteWithPublication 1 (fun _ => .ok 7) (fun _ _ => .ok (1/2))
      1 2 1 = .ok 0 ∧
    poissonFiniteWithPublication 1 (fun _ => .ok 7) (fun _ _ => .ok 1)
      1 2 1 = .ok 1 := by
  constructor
  · have nonnegative : ¬ ((1/2 : Rat) < 0) := by native_decide
    have tiny : (1/2 : Rat) < 1 := by native_decide
    have published : poissonPositiveCountPublication 1 (1/2) = 0 := by
      simp [poissonPositiveCountPublication, nonnegative, tiny]
    change (Except.ok (poissonPositiveCountPublication 1 (1/2)) :
      Except WorksheetErrorCode Rat) = .ok 0
    rw [published]
  · rfl

/-- W111 finite probability and admitted integer-domain slice. Integer
conversion itself and interior numerical kernels remain separate obligations. -/
def binomialInverseDomain (trials : Int) (probability alpha : Rat) : Bool :=
  decide (0 ≤ trials ∧ 0 < probability ∧ probability < 1 ∧ 0 < alpha ∧ alpha < 1)

def negativeBinomialDomain (failures successes : Int) (probability : Rat) : Bool :=
  decide (0 ≤ failures ∧ 0 < successes ∧ 0 < probability ∧ probability < 1)

/-- Raw negative hypergeometric parameters are rejected before truncation;
positive support/population comparisons occur after integer conversion. -/
def hypergeometricRawNonnegative (k draws successes population : Rat) : Bool :=
  decide (0 ≤ k ∧ 0 ≤ draws ∧ 0 ≤ successes ∧ 0 ≤ population)

theorem hypergeometric_negative_fraction_rejected :
    hypergeometricRawNonnegative 0 (-(1/10)) 3 7 = false := by native_decide

/-- `none` delegates to the existing interior numerical backend; impossible
outcomes have exact probabilities, distinct from invalid population parameters. -/
def hypergeometricIntegerSupport (k draws successes population : Int) (cumulative : Bool) :
    Except WorksheetErrorCode (Option Rat) :=
  if k < 0 || draws < 0 || successes < 0 || population < 0 ||
      successes > population || draws > population then .error .num
  else if k < max 0 (draws + successes - population) then .ok (some 0)
  else if k > min draws successes then .ok (some (if cumulative then 1 else 0))
  else .ok none

theorem binomial_inverse_open_endpoints :
    binomialInverseDomain 2 0 (1/2) = false ∧
    binomialInverseDomain 2 1 (1/2) = false ∧
    binomialInverseDomain 2 (1/2) 0 = false ∧
    binomialInverseDomain 2 (1/2) 1 = false := by native_decide

theorem negative_binomial_open_probability :
    negativeBinomialDomain 0 2 0 = false ∧
    negativeBinomialDomain 0 2 1 = false := by native_decide

theorem hypergeometric_impossible_outcome_is_probability :
    hypergeometricIntegerSupport 1 0 2 4 false = .ok (some 0) ∧
    hypergeometricIntegerSupport 1 0 2 4 true = .ok (some 1) ∧
    hypergeometricIntegerSupport 0 3 3 4 true = .ok (some 0) := by
  exact ⟨rfl, rfl, rfl⟩

theorem expon_missing_numeric_and_flag_arguments :
    exponDistPreparedArguments [.missingArg, .number 1, .missingArg] = .ok ([0, 1], false) := by rfl

theorem expon_cumulative_logical_text_binding :
    exponDistPreparedArguments [.number 0, .number 1, .text "TRUE"] = .ok ([0, 1], true) := by rfl

end OxFunc.Functions

