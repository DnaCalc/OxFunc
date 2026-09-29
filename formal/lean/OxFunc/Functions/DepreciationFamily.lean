import OxFunc.FunctionCore
import OxFunc.CoercionPrimitives
import OxFunc.Functions.DistributionArguments

namespace OxFunc.Functions
open OxFunc

private instance instDecidableEqExceptDepreciation [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def depreciationBase : FunctionMeta :=
{ functionId := "FUNC.DEPRECIATION_BASE", arity := Arity.exact 1,
  determinism := DeterminismClass.deterministic, volatility := VolatilityClass.nonvolatile,
  hostInteraction := HostInteractionClass.none, threadSafety := ThreadSafetyClass.safePure,
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter,
  coercionLiftProfile := CoercionLiftProfile.custom, kernelSignatureClass := KernelSignatureClass.custom,
  fecDependencyProfile := FecDependencyProfile.none,
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly }

def slnMeta : FunctionMeta :=
have __src := depreciationBase;
{ functionId := "FUNC.SLN", arity := Arity.exact 3, determinism := __src.determinism,
  volatility := __src.volatility, hostInteraction := __src.hostInteraction, threadSafety := __src.threadSafety,
  argPreparationProfile := __src.argPreparationProfile, coercionLiftProfile := __src.coercionLiftProfile,
  kernelSignatureClass := __src.kernelSignatureClass, fecDependencyProfile := __src.fecDependencyProfile,
  surfaceFecDependencyProfile := __src.surfaceFecDependencyProfile }

def sydMeta : FunctionMeta :=
have __src := depreciationBase;
{ functionId := "FUNC.SYD", arity := Arity.exact 4, determinism := __src.determinism,
  volatility := __src.volatility, hostInteraction := __src.hostInteraction, threadSafety := __src.threadSafety,
  argPreparationProfile := __src.argPreparationProfile, coercionLiftProfile := __src.coercionLiftProfile,
  kernelSignatureClass := __src.kernelSignatureClass, fecDependencyProfile := __src.fecDependencyProfile,
  surfaceFecDependencyProfile := __src.surfaceFecDependencyProfile }

def dbMeta : FunctionMeta :=
have __src := depreciationBase;
{ functionId := "FUNC.DB", arity := { min := 4, max := 5 }, determinism := __src.determinism,
  volatility := __src.volatility, hostInteraction := __src.hostInteraction, threadSafety := __src.threadSafety,
  argPreparationProfile := __src.argPreparationProfile, coercionLiftProfile := __src.coercionLiftProfile,
  kernelSignatureClass := __src.kernelSignatureClass, fecDependencyProfile := __src.fecDependencyProfile,
  surfaceFecDependencyProfile := __src.surfaceFecDependencyProfile }

def ddbMeta : FunctionMeta :=
have __src := depreciationBase;
{ functionId := "FUNC.DDB", arity := { min := 4, max := 5 }, determinism := __src.determinism,
  volatility := __src.volatility, hostInteraction := __src.hostInteraction, threadSafety := __src.threadSafety,
  argPreparationProfile := __src.argPreparationProfile, coercionLiftProfile := __src.coercionLiftProfile,
  kernelSignatureClass := __src.kernelSignatureClass, fecDependencyProfile := __src.fecDependencyProfile,
  surfaceFecDependencyProfile := __src.surfaceFecDependencyProfile }

def vdbMeta : FunctionMeta :=
have __src := depreciationBase;
{ functionId := "FUNC.VDB", arity := { min := 5, max := 7 }, determinism := __src.determinism,
  volatility := __src.volatility, hostInteraction := __src.hostInteraction, threadSafety := __src.threadSafety,
  argPreparationProfile := __src.argPreparationProfile, coercionLiftProfile := __src.coercionLiftProfile,
  kernelSignatureClass := __src.kernelSignatureClass, fecDependencyProfile := __src.fecDependencyProfile,
  surfaceFecDependencyProfile := __src.surfaceFecDependencyProfile }

/-- SLN admits either sign and nonzero life. Rust stores the difference and uses x87 division before numeric publication. -/
def slnKernelModel : Rat → Rat → Rat → Except WorksheetErrorCode Rat :=
fun cost salvage life =>
  if life = 0 then Except.error WorksheetErrorCode.div0 else Except.ok ((cost - salvage) / life)

def prepareSlnMissing : CoercionInput → CoercionInput :=
fun input =>
  match input with
  | CoercionInput.missingArg => CoercionInput.number 0
  | other => other

inductive SlnPublicationRoute where
  | numericError | divisionByZero | positiveZero | storedQuotient
  deriving DecidableEq, Repr

/-- Ordered SLN publication: finite inputs, zero divisor, finite result, then subnormal flush. -/
def slnPublicationRoute : Bool → Bool → Bool → Bool → SlnPublicationRoute :=
fun inputsFinite lifeZero resultFinite resultZeroOrSubnormal =>
  if (!inputsFinite) = true then SlnPublicationRoute.numericError
  else
    if lifeZero = true then SlnPublicationRoute.divisionByZero
    else
      if (!resultFinite) = true then SlnPublicationRoute.numericError
      else
        if resultZeroOrSubnormal = true then SlnPublicationRoute.positiveZero
        else SlnPublicationRoute.storedQuotient

theorem slnKernelModel_zero_life : ∀ (cost salvage : Rat),
  slnKernelModel cost salvage 0 = Except.error WorksheetErrorCode.div0 := by intro cost salvage; simp [slnKernelModel]

theorem slnKernelModel_nonzero_life : ∀ (cost salvage life : Rat),
  life ≠ 0 → slnKernelModel cost salvage life = Except.ok ((cost - salvage) / life) := by intro cost salvage life h; simp [slnKernelModel, h]

theorem prepareSlnMissing_coerces_to_zero : coerceToNumber
    (prepareSlnMissing CoercionInput.missingArg) =
  Except.ok 0 := by native_decide

theorem slnPublicationRoute_witnesses : slnPublicationRoute true true false false =
    SlnPublicationRoute.divisionByZero ∧
  slnPublicationRoute true false false false = SlnPublicationRoute.numericError ∧
    slnPublicationRoute true false true true = SlnPublicationRoute.positiveZero ∧
      slnPublicationRoute true false true false =
        SlnPublicationRoute.storedQuotient := by native_decide

/-- Ordinary nonnegative truncated-count slice; fractional lifetime is retained. The host-count boundary model below records the full observed count route. -/
def dbAdmissionModel : Rat → Rat → Rat → Rat → Nat → Nat → Bool :=
fun cost salvage life originalPeriod wholePeriod month =>
  decide
    (0 ≤ cost ∧
      0 ≤ salvage ∧
        0 < life ∧ 0 < originalPeriod ∧ 1 ≤ month ∧ month ≤ 12 ∧ ↑wholePeriod ≤ life + if month < 12 then 1 else 0)

/-- Nonnegative finite count saturates to u32 then reinterprets as signed. Admission rejects negative original inputs before this conversion. -/
def dbHostCountModel : Rat → Int :=
fun value =>
  have n := min (value.num.toNat / value.den) 4294967295;
  if n < 2147483648 then ↑n else ↑n - 4294967296

def dbObservedAdmissionModel : Rat → Rat → Rat → Rat → Rat → Bool :=
fun cost salvage life period month =>
  have l := dbHostCountModel life;
  have p := dbHostCountModel period;
  have m := dbHostCountModel month;
  decide (0 ≤ cost ∧ 0 ≤ salvage ∧ 0 < life ∧ 0 < period ∧ 1 ≤ m ∧ m ≤ 12 ∧ p ≤ l + if m < 12 then 1 else 0)

theorem dbHostCountModel_boundary_observations : dbHostCountModel 2147483647 =
    2147483647 ∧
  dbHostCountModel 2147483648 = -2147483648 ∧
    dbHostCountModel 4294967295 = -1 ∧ dbHostCountModel 100000000000000000000 = -1 := by native_decide

theorem dbObservedAdmissionModel_boundary_observations : dbObservedAdmissionModel 1000
      0 2147483648 (1 / 2) 6 =
    false ∧
  dbObservedAdmissionModel 1000 0 4294967296 (1 / 2) 6 = true ∧
    dbObservedAdmissionModel 1000 0 4294967296 1 6 = false ∧
      dbObservedAdmissionModel 1000 0 4294967296 (1 / 2) 12 = false := by native_decide

def dbBookSteps (rate : Rat) (n : Nat) (book dep : Rat) : Rat × Rat :=
  match n with
  | 0 => (book, dep)
  | n+1 =>
      let nextDep := book * rate
      dbBookSteps rate n (book - nextDep) nextDep
termination_by n

def dbScheduleModel : Rat → Rat → Nat → Nat → Nat → Rat :=
fun cost rate month wholePeriod wholeLife =>
  have first := cost * rate * (↑month / 12);
  match dbBookSteps rate (min wholePeriod wholeLife - 1) (cost - first) first with
  | (book, dep) => if month < 12 ∧ wholePeriod = wholeLife + 1 then book / 12 * rate * (12 - ↑month) else dep

/-- DB ratio and internal POWER input/output stages publish nonfinite or subnormal values as zero. -/
def dbInternalStageModel : Rat → Bool → Bool → Rat :=
fun value finite subnormal => if (!finite || subnormal) = true then 0 else value

theorem dbInternalStageModel_observations : ∀ (value : Rat),
  dbInternalStageModel value false false = 0 ∧
    dbInternalStageModel value true true = 0 ∧
      dbInternalStageModel value true false = value := by intro value; simp [dbInternalStageModel]

/-- The internal integer-power path uses a u32::MAX sentinel, distinct from the worksheet POWER wrapper. -/
def dbIntegerPowerPathModel : Rat → Bool :=
fun exponent => decide (exponent.den = 1 ∧ 0 ≤ exponent ∧ exponent < 4294967295)

theorem dbIntegerPowerPathModel_upper_boundary : dbIntegerPowerPathModel 4294967294 =
    true ∧
  dbIntegerPowerPathModel 4294967295 = false ∧
    dbIntegerPowerPathModel (5 / 2) = false := by native_decide

def dbPowerPublicationModel : Rat → Bool → Bool → Bool → Except WorksheetErrorCode Rat :=
fun value integerPath finite subnormal =>
  if (!finite) = true then if integerPath = true then Except.ok 0 else Except.error WorksheetErrorCode.num
  else Except.ok (if subnormal = true then 0 else value)

def dbAmountPublicationModel : Rat → Bool → Rat :=
fun amount subnormal => if subnormal = true then 0 else amount

theorem dbAmountPublicationModel_observations : ∀ (amount : Rat),
  dbAmountPublicationModel amount true = 0 ∧
    dbAmountPublicationModel amount false = amount := by intro amount; simp [dbAmountPublicationModel]

def dbRateNanModel : Rat → Bool → Rat :=
fun roundedRate powerNaN => if powerNaN = true then 0 else roundedRate

theorem dbRateNanModel_publication : ∀ (rate : Rat),
  dbRateNanModel rate true = 0 ∧ dbRateNanModel rate false = rate := by intro rate; simp [dbRateNanModel]

theorem dbPowerPublicationModel_overflow_branches : ∀ (value : Rat),
  dbPowerPublicationModel value true false false = Except.ok 0 ∧
    dbPowerPublicationModel value false false false = Except.error WorksheetErrorCode.num := by intro value; simp [dbPowerPublicationModel]

/-- Rational DDB substrate; Rust supplies the witnessed power and rounding graph. This model does not prove IEEE identity. -/
def ddbKernelModel : (Rat → Rat → Rat) →
  Rat → Rat → Rat → Rat → Rat → Except WorksheetErrorCode Rat :=
fun power cost salvage life period factor =>
  if cost < 0 ∨ salvage < 0 ∨ life ≤ 0 ∨ period ≤ 0 ∨ life < period ∨ factor ≤ 0 then
    Except.error WorksheetErrorCode.num
  else
    if cost ≤ salvage then Except.ok 0
    else
      have rate := factor / life;
      if 1 ≤ rate then Except.ok (if period ≤ 1 then cost - salvage else 0)
      else
        have book := cost * power (1 - rate) (max 0 (period - 1));
        Except.ok (max 0 (min (book * rate) (book - salvage)))

theorem db_fractional_period_admission : dbAdmissionModel 1000 100 (1 / 2) (1 / 2) 0
      1 =
    true ∧
  dbAdmissionModel 1000 100 (1 / 2) 1 1 1 = true ∧
    dbAdmissionModel 1000 100 10 11 11 12 = false := by native_decide

theorem db_book_schedule_witnesses : dbScheduleModel 1000 (99 / 100) 1 0 0 = 165 / 2 ∧
  dbScheduleModel 1000 (99 / 100) 1 1 0 = 666105 / 800 := by native_decide

theorem ddb_rate_and_period_branches : ddbKernelModel (fun _ _ => 1) 1000 100 10
      (1 / 2) 2 =
    Except.ok 200 ∧
  ddbKernelModel (fun _ _ => 1) 1000 100 10 1 20 = Except.ok 900 ∧
    ddbKernelModel (fun _ _ => 1) 1000 100 10 2 20 = Except.ok 0 ∧
      ddbKernelModel (fun _ _ => 1) 1000 2000 10 1 2 = Except.ok 0 := by native_decide

def ddbRatePublicationModel : Rat → Bool → Rat :=
fun rate subnormal => if subnormal = true then 0 else rate

theorem ddbRatePublicationModel_observations : ∀ (rate : Rat),
  ddbRatePublicationModel rate true = 0 ∧ ddbRatePublicationModel rate false = rate := by intro rate; simp [ddbRatePublicationModel]

/-- Two independently published partial products precede subtraction. The final difference can preserve a nonzero subnormal result. -/
def vdbNoSwitchYearModel : (Rat → Rat → Rat) → (Rat → Rat → Rat) → Rat → Rat → Rat → Rat → Rat :=
fun mul sub annual startBound endBound year =>
  sub (mul annual (1 - max 0 (year + 1 - endBound))) (mul annual (max 0 (startBound - year)))

theorem vdbNoSwitchYearModel_rational_witness : vdbNoSwitchYearModel
      (fun x1 x2 => x1 * x2) (fun x1 x2 => x1 - x2) 900 (1 / 2) 1 0 =
    450 ∧
  vdbNoSwitchYearModel (fun x1 x2 => x1 * x2) (fun x1 x2 => x1 - x2) (2000 / 3) (1 / 4) (3 / 8) 0 =
    250 / 3 := by native_decide

theorem depreciation_family_profiles : slnMeta.argPreparationProfile =
    ArgPreparationProfile.valuesOnlyPreAdapter ∧
  sydMeta.arity = Arity.exact 4 ∧
    dbMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly ∧
      ddbMeta.threadSafety = ThreadSafetyClass.safePure ∧
        vdbMeta.arity.max = 7 := by native_decide

/-- Financial native lifting binds the shared positional prepared-argument
substrate. Rust supplies per-coordinate groups, retaining absent coordinates
as `.error .na` in their original argument position. Optional defaults are
selected before this model; rectangular construction is separately tested. -/
def depreciationPreparedNumericGroups (count : Nat)
    (groups : List (List CoercionInput)) :
    List (Except WorksheetErrorCode (List Rat)) :=
  groups.map (distributionNumericPrepared count)

theorem depreciation_padding_preserves_error_order :
    depreciationPreparedNumericGroups 3
      [[.error .ref, .number 1, .error .na],
       [.error .na, .number 1, .error .ref],
       [.missingArg, .number 1, .number 2]] =
      [.error .ref, .error .na, .ok [0, 1, 2]] := by rfl

/-- SYD admits either cost sign. Its period comparison publishes a subnormal
positive difference as zero; the Boolean names that IEEE substrate result. -/
def sydAdmissionModel (salvage life period : Rat)
    (periodExcessPublishesPositive : Bool) : Bool :=
  decide (0 ≤ salvage ∧ 0 < life ∧ 0 < period) && !periodExcessPublishesPositive

/-- Ordered SYD arithmetic binds the witnessed store and stage publication
operations. This does not identify rational arithmetic with IEEE arithmetic. -/
def sydStoredStages (add sub mul div : Rat → Rat → Rat)
    (publishStage : Rat → Rat) (cost salvage life period : Rat) : Rat × Rat :=
  let lifePlusOne := add life 1
  let denominator := publishStage (div (mul life lifePlusOne) 2)
  let basis := publishStage (sub cost salvage)
  let numerator := mul basis (sub lifePlusOne period)
  (numerator, denominator)

theorem sydAdmissionModel_observations :
    sydAdmissionModel 0 10 1 false = true ∧
    sydAdmissionModel (-20) 10 1 false = false ∧
    sydAdmissionModel 0 10 0 false = false ∧
    sydAdmissionModel 0 10 11 true = false := by native_decide

theorem sydStoredStages_rational_witness :
    sydStoredStages (· + ·) (· - ·) (· * ·) (· / ·) id (-100) 0 10 1 =
      (-1000, 55) := by native_decide

/-- Denominator publication precedes basis overflow. A finite denominator
exposes basis overflow as NUM; denominator overflow publishes zero first. -/
def sydPublicationRoute (denominatorZero basisFinite resultFinite resultSubnormal : Bool) : SlnPublicationRoute :=
  if denominatorZero then .divisionByZero
  else if !basisFinite || !resultFinite then .numericError
  else if resultSubnormal then .positiveZero
  else .storedQuotient

theorem sydPublicationRoute_error_order :
    sydPublicationRoute true false false false = .divisionByZero ∧
    sydPublicationRoute false false false false = .numericError ∧
    sydPublicationRoute false true true true = .positiveZero := by decide

/-- VDB's observed switch has an inclusive absolute comparison boundary.
`minimumNormal` binds to the binary64 minimum positive normal. Stored amounts
have already passed their numeric publication. This rule is local to VDB. -/
def vdbSwitchExceeds (minimumNormal straight declining : Rat) : Bool :=
  decide (straight > declining ∧ straight - declining ≥ minimumNormal / 16)

inductive VdbAmountRoute where
  | clipped | straightLine | declining
  deriving DecidableEq, Repr

/-- Clipping precedes the first switch. IEEE finite checks execute before this
admitted route model, including the unclamped remaining-life division. -/
def vdbAmountRoute (alreadySwitched : Bool)
    (minimumNormal straight declining available : Rat) : VdbAmountRoute :=
  if !alreadySwitched && decide (declining > available) then .clipped
  else if vdbSwitchExceeds minimumNormal straight (min declining available) then .straightLine
  else .declining

theorem vdbSwitchExceeds_inclusive_boundary :
    vdbSwitchExceeds 16 33 32 = true ∧
    vdbSwitchExceeds 16 (65/2) 32 = false ∧
    vdbSwitchExceeds 16 32 32 = false := by native_decide

theorem vdbAmountRoute_clipping_order :
    vdbAmountRoute false 16 40 30 20 = .clipped ∧
    vdbAmountRoute true 16 40 30 20 = .straightLine := by native_decide

/-- Whole-year amounts bypass multiplication publication. Fractional products
are published before addition; a final total can preserve a subnormal value. -/
def vdbSwitchedPartModel (mul : Rat → Rat → Rat) (publish : Rat → Rat)
    (amount take : Rat) : Rat :=
  if take = 1 then amount else publish (mul amount take)

theorem vdbSwitchedPartModel_whole_preserves_amount
    (mul : Rat → Rat → Rat) (publish : Rat → Rat) (amount : Rat) :
    vdbSwitchedPartModel mul publish amount 1 = amount := by
  simp [vdbSwitchedPartModel]

/-- Advance book subtraction binds to an x87 store; requested-period book
subtraction binds to ordinary binary64 in Rust. Rat is only the substrate. -/
def vdbStoredBookModel (storedSub : Rat → Rat → Rat)
    (advancing : Bool) (book part : Rat) : Rat :=
  if advancing then storedSub book part else book - part

theorem vdbStoredBookModel_routes (storedSub : Rat → Rat → Rat) (book part : Rat) :
    vdbStoredBookModel storedSub true book part = storedSub book part ∧
    vdbStoredBookModel storedSub false book part = book - part := by
  simp [vdbStoredBookModel]

/-- Terminate from the stored remaining length, not a newly rounded cursor.
This bounded operation binding does not establish huge-period progress. -/
def vdbRequestedStepFinal (remaining : Rat) (switched : Bool) : Bool :=
  decide (remaining ≤ 1) || switched

theorem vdbRequestedStepFinal_observations :
    vdbRequestedStepFinal 1 false = true ∧
    vdbRequestedStepFinal (3/2) false = false ∧
    vdbRequestedStepFinal (3/2) true = true := by native_decide

end OxFunc.Functions
