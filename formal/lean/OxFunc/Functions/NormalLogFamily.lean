import OxFunc.FunctionCore
import OxFunc.NumericPublication
import OxFunc.Functions.DistributionArguments

namespace OxFunc.Functions

open OxFunc

def normalLogBaseMeta : FunctionMeta := {
  functionId := "FUNC.NORMAL_LOG_BASE"
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

def confidenceMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.CONFIDENCE"
  arity := Arity.exact 3
}

def confidenceNormMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.CONFIDENCE.NORM"
  arity := Arity.exact 3
}

def lognormDistMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.LOGNORM.DIST"
  arity := Arity.exact 4
}

def lognormInvMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.LOGNORM.INV"
  arity := Arity.exact 3
}

def lognormdistMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.LOGNORMDIST"
  arity := Arity.exact 3
}

def normDistMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.NORM.DIST"
  arity := Arity.exact 4
}

def normInvMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.NORM.INV"
  arity := Arity.exact 3
}

def normSDistMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.NORM.S.DIST"
  arity := Arity.exact 2
}

def normSInvMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.NORM.S.INV"
}

def normdistMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.NORMDIST"
  arity := Arity.exact 4
}

def norminvMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.NORMINV"
  arity := Arity.exact 3
}

def normsdistMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.NORMSDIST"
}

def normsinvMeta : FunctionMeta := {
  normalLogBaseMeta with
  functionId := "FUNC.NORMSINV"
}

theorem normalLogMeta_profiles :
    confidenceMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ confidenceNormMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ lognormDistMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ lognormInvMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ lognormdistMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ normDistMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ normInvMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ normSDistMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ normSInvMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ normdistMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ norminvMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ normsdistMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ normsinvMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [
    normalLogBaseMeta, confidenceMeta, confidenceNormMeta, lognormDistMeta, lognormInvMeta,
    lognormdistMeta, normDistMeta, normInvMeta, normSDistMeta, normSInvMeta, normdistMeta,
    norminvMeta, normsdistMeta, normsinvMeta
  ]

def normDistPreparedArguments := distributionPrepared 3
def normSDistPreparedArguments := distributionPrepared 1

def confidencePreparedArguments := distributionNumericPrepared 3
def normInvPreparedArguments := distributionNumericPrepared 3
def lognormInvPreparedArguments := distributionNumericPrepared 3
def lognormdistPreparedArguments := distributionNumericPrepared 3
def lognormDistPreparedArguments := distributionPrepared 3

theorem adjacent_normal_missing_mean_binding :
    normInvPreparedArguments [.number (1/2), .missingArg, .number 1] = .ok [1/2, 0, 1] ∧
    lognormInvPreparedArguments [.number (1/2), .missingArg, .number 1] = .ok [1/2, 0, 1] := by
  exact ⟨rfl, rfl⟩

theorem lognormal_cumulative_text_binding :
    lognormDistPreparedArguments [.number 1, .number 0, .number 1, .text "false"] = .ok ([1, 0, 1], false) ∧
    lognormDistPreparedArguments [.number 1, .number 0, .number 1, .text "0"] = .error .value := by
  exact ⟨rfl, rfl⟩

theorem norm_s_missing_density_arguments :
    normSDistPreparedArguments [.missingArg, .missingArg] = .ok ([0], false) := by rfl

theorem norm_distribution_cumulative_numeric_text_rejected :
    normDistPreparedArguments [.number 0, .number 0, .number 1, .text "1"] = .error .value := by rfl

/-- Density-only W111 expression graph. Supplied arithmetic primitives publish
binary64 after RN64 operations; the EXP primitive remains explicitly supplied.
The CDF path is not replaced by this graph. -/
def normDensityWithPrimitives (sub div mul : Float → Float → Float)
    (exp : Float → Float) (x mean sigma : Float) : Except WorksheetErrorCode Float :=
  if sigma ≤ 0 || !sigma.isFinite then .error .num
  else
    let z := div (sub x mean) sigma
    let square := mul z z
    if !square.isFinite then .error .num
    else
      let exponential := exp (-(square / 2))
      .ok (NumericPublication.flushTiny
        (mul (div exponential sigma) (Float.ofBits 0x3fd9884533d43651)))

theorem norm_density_zero_sigma_rejected :
    (normDensityWithPrimitives Float.sub Float.div Float.mul Float.exp 0 0 0).toOption = none := by
  native_decide

theorem norm_density_unit_center_binding :
    ((normDensityWithPrimitives Float.sub Float.div Float.mul Float.exp 0 0 1).toOption.map Float.toBits)
      = some 0x3fd9884533d43651 := by
  native_decide

theorem norm_density_square_overflow_binding :
    (normDensityWithPrimitives Float.sub Float.div Float.mul Float.exp 1.0e308 0 1).toOption = none := by
  native_decide

/-- W111 ordinary CDF wrapper. `mul` publishes RN64 multiplication to binary64;
`erfc` is an explicitly supplied dependency whose local kernel remains open.
The separate GAUSS tiny branch is outside this wrapper. -/
def normalCdfWithPrimitives (mul : Float → Float → Float) (erfc : Float → Float)
    (x : Float) : Float :=
  if x == 0 then 0.5
  else
    let z := mul x.abs (Float.ofBits 0x3fe6a09e667f3bcd)
    let halfQ := erfc z / 2
    NumericPublication.flushTiny (if x < 0 then halfQ else 1 - halfQ)

theorem normal_cdf_center_binding :
    (normalCdfWithPrimitives Float.mul (fun _ => 7) 0).toBits =
      (0.5 : Float).toBits := by native_decide

/-- Bind the public intermediate observation at the original staging separator;
this tests wrapper dataflow, without claiming a Lean ERFC implementation. -/
theorem normal_cdf_staged_z_public_dependency_binding :
    (normalCdfWithPrimitives (fun _ _ => Float.ofBits 0x4036a71b83b8015e)
      (fun z => if z.toBits == 0x4036a71b83b8015e then Float.ofBits 0x115478e2d4c364b8 else 0)
      (Float.ofBits 0xc040049695c7ac92)).toBits = 0x114478e2d4c364b8 := by native_decide

end OxFunc.Functions
