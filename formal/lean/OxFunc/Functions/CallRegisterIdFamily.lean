import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

def callMeta : FunctionMeta := {
  functionId := "FUNC.CALL"
  arity := { min := 1, max := 255 }
  determinism := DeterminismClass.externalEventDependent
  volatility := VolatilityClass.volatileContextual
  hostInteraction := HostInteractionClass.applicationState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.externalProvider
  surfaceFecDependencyProfile := FecDependencyProfile.composite
}

def registerIdMeta : FunctionMeta := {
  functionId := "FUNC.REGISTER.ID"
  arity := { min := 2, max := 3 }
  determinism := DeterminismClass.externalEventDependent
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.applicationState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.externalProvider
  surfaceFecDependencyProfile := FecDependencyProfile.externalProvider
}

inductive RegisteredProcedureSpec where
  | name (value : String)
  | ordinal (value : Int)
  deriving DecidableEq, Repr

inductive RegisteredExternalOriginKind where
  | worksheetRegisterId
  | hostRegisteredExternal
  deriving DecidableEq, Repr

structure RegisterIdRequest where
  libraryName : String
  procedure : RegisteredProcedureSpec
  declaredTypeText : Option String
  deriving DecidableEq, Repr

structure RegisteredExternalDescriptor where
  stableRegistrationId : String
  registerId : Int
  originKind : RegisteredExternalOriginKind
  displayName : Option String
  libraryName : String
  procedure : RegisteredProcedureSpec
  declaredTypeText : Option String
  deriving DecidableEq, Repr

inductive RegisteredExternalTarget where
  | registerId (value : Int)
  | direct (request : RegisterIdRequest)
  deriving DecidableEq, Repr

structure RegisteredExternalCallRequest where
  target : RegisteredExternalTarget
  scalarArgs : List Int
  deriving DecidableEq, Repr

def resolveRegisterIdSeed : RegisterIdRequest → Option RegisteredExternalDescriptor
  | {
      libraryName := "Kernel32",
      procedure := .name "GetTickCount",
      declaredTypeText := some "J!"
    } =>
      some {
        stableRegistrationId := "REG.tickcount"
        registerId := -1439170560
        originKind := .worksheetRegisterId
        displayName := some "GetTickCount"
        libraryName := "Kernel32"
        procedure := .name "GetTickCount"
        declaredTypeText := some "J!"
      }
  | {
      libraryName := "Kernel32",
      procedure := .name "GetTickCount",
      declaredTypeText := none
    } =>
      some {
        stableRegistrationId := "REG.tickcount"
        registerId := -1439170560
        originKind := .worksheetRegisterId
        displayName := some "GetTickCount"
        libraryName := "Kernel32"
        procedure := .name "GetTickCount"
        declaredTypeText := none
      }
  | {
      libraryName := "Kernel32",
      procedure := .name "MulDiv",
      declaredTypeText := some "JJJJ"
    } =>
      some {
        stableRegistrationId := "REG.muldiv"
        registerId := -1995046912
        originKind := .worksheetRegisterId
        displayName := some "MulDiv"
        libraryName := "Kernel32"
        procedure := .name "MulDiv"
        declaredTypeText := some "JJJJ"
      }
  | _ => none

def lookupRegisteredExternalSeed : Int → Option RegisteredExternalDescriptor
  | -1439170560 =>
      resolveRegisterIdSeed {
        libraryName := "Kernel32"
        procedure := .name "GetTickCount"
        declaredTypeText := some "J!"
      }
  | -1995046912 =>
      resolveRegisterIdSeed {
        libraryName := "Kernel32"
        procedure := .name "MulDiv"
        declaredTypeText := some "JJJJ"
      }
  | _ => none

def invokeRegisteredExternalSeed :
    RegisteredExternalDescriptor → List Int → Option Int
  | { stableRegistrationId := "REG.tickcount", .. }, [] => some 827899000
  | { stableRegistrationId := "REG.muldiv", .. }, [a, b, c] =>
      if c = 0 then none else some ((a * b) / c)
  | _, _ => none

def evalRegisterIdSeed (request : RegisterIdRequest) : Option Int := do
  let descriptor ← resolveRegisterIdSeed request
  pure descriptor.registerId

def evalCallSeed (request : RegisteredExternalCallRequest) : Option Int := do
  let descriptor ← match request.target with
    | .registerId n => lookupRegisteredExternalSeed n
    | .direct registerRequest => resolveRegisterIdSeed registerRequest
  invokeRegisteredExternalSeed descriptor request.scalarArgs

theorem callRegisterIdFamily_meta_profiles :
    callMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ callMeta.surfaceFecDependencyProfile = FecDependencyProfile.composite
    ∧ registerIdMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ registerIdMeta.fecDependencyProfile = FecDependencyProfile.externalProvider := by
  simp [callMeta, registerIdMeta]

theorem register_id_seed_gettickcount :
    evalRegisterIdSeed {
      libraryName := "Kernel32"
      procedure := .name "GetTickCount"
      declaredTypeText := some "J!"
    } = some (-1439170560) := by
  native_decide

theorem call_seed_by_register_id_gettickcount :
    evalCallSeed {
      target := .registerId (-1439170560)
      scalarArgs := []
    } = some 827899000 := by
  native_decide

theorem call_seed_direct_muldiv :
    evalCallSeed {
      target := .direct {
        libraryName := "Kernel32"
        procedure := .name "MulDiv"
        declaredTypeText := some "JJJJ"
      }
      scalarArgs := [6, 7, 3]
    } = some 14 := by
  native_decide

theorem register_id_seed_missing_type_text_succeeds_for_seeded_zero_arg_lane :
    evalRegisterIdSeed {
      libraryName := "Kernel32"
      procedure := .name "GetTickCount"
      declaredTypeText := none
    } = some (-1439170560) := by
  native_decide

theorem call_seed_missing_type_text_succeeds_for_seeded_zero_arg_lane :
    evalCallSeed {
      target := .direct {
        libraryName := "Kernel32"
        procedure := .name "GetTickCount"
        declaredTypeText := none
      }
      scalarArgs := []
    } = some 827899000 := by
  native_decide

end OxFunc.Functions
