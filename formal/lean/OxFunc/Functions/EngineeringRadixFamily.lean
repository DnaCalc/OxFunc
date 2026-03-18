import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def engineeringRadixTextMeta : FunctionMeta := {
  functionId := "FUNC.ENGINEERING_RADIX_TEXT"
  arity := { min := 1, max := 2 }
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

def engineeringRadixDecimalMeta : FunctionMeta := {
  engineeringRadixTextMeta with
    functionId := "FUNC.ENGINEERING_RADIX_DECIMAL"
    arity := Arity.exact 1
}

def dec2binMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.DEC2BIN" }
def dec2hexMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.DEC2HEX" }
def dec2octMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.DEC2OCT" }
def bin2decMeta : FunctionMeta := { engineeringRadixDecimalMeta with functionId := "FUNC.BIN2DEC" }
def bin2hexMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.BIN2HEX" }
def bin2octMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.BIN2OCT" }
def hex2binMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.HEX2BIN" }
def hex2decMeta : FunctionMeta := { engineeringRadixDecimalMeta with functionId := "FUNC.HEX2DEC" }
def hex2octMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.HEX2OCT" }
def oct2binMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.OCT2BIN" }
def oct2decMeta : FunctionMeta := { engineeringRadixDecimalMeta with functionId := "FUNC.OCT2DEC" }
def oct2hexMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.OCT2HEX" }

theorem engineeringRadixFamily_profiles :
    dec2binMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ dec2hexMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ dec2octMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ bin2decMeta.arity = Arity.exact 1
    ∧ bin2hexMeta.arity.min = 1
    ∧ bin2octMeta.arity.max = 2
    ∧ hex2binMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ hex2decMeta.arity = Arity.exact 1
    ∧ hex2octMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ oct2binMeta.hostInteraction = HostInteractionClass.none
    ∧ oct2decMeta.arity = Arity.exact 1
    ∧ oct2hexMeta.volatility = VolatilityClass.nonvolatile := by
  simp [
    engineeringRadixTextMeta,
    engineeringRadixDecimalMeta,
    dec2binMeta,
    dec2hexMeta,
    dec2octMeta,
    bin2decMeta,
    bin2hexMeta,
    bin2octMeta,
    hex2binMeta,
    hex2decMeta,
    hex2octMeta,
    oct2binMeta,
    oct2decMeta,
    oct2hexMeta
  ]

theorem engineeringRadixFamily_decimal_vs_text_arity :
    bin2decMeta.arity = Arity.exact 1
    ∧ hex2decMeta.arity = Arity.exact 1
    ∧ oct2decMeta.arity = Arity.exact 1
    ∧ dec2binMeta.arity = { min := 1, max := 2 }
    ∧ bin2hexMeta.arity = { min := 1, max := 2 }
    ∧ hex2octMeta.arity = { min := 1, max := 2 } := by
  simp [
    engineeringRadixTextMeta,
    engineeringRadixDecimalMeta,
    dec2binMeta,
    bin2decMeta,
    bin2hexMeta,
    hex2decMeta,
    hex2octMeta,
    oct2decMeta
  ]

end OxFunc.Functions
