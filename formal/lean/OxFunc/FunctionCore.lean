import OxFunc.Types

namespace OxFunc

inductive DeterminismClass where
  | deterministic
  | pseudoRandom
  | timeDependent
  | externalEventDependent
  deriving DecidableEq, Repr

inductive VolatilityClass where
  | nonvolatile
  | volatileFull
  | volatileContextual
  deriving DecidableEq, Repr

inductive HostInteractionClass where
  | none
  | workbookState
  | applicationState
  | environmentState
  | externalProvider
  deriving DecidableEq, Repr

inductive ThreadSafetyClass where
  | safePure
  | hostSerialized
  | notThreadSafe
  deriving DecidableEq, Repr

inductive FecDependencyProfile where
  | none
  | refOnly
  | callerContext
  | timeProvider
  | randomProvider
  | externalProvider
  | localeProfile
  | composite
  deriving DecidableEq, Repr

structure Arity where
  min : Nat
  max : Nat
  deriving DecidableEq, Repr

def Arity.exact (n : Nat) : Arity := { min := n, max := n }

def Arity.accepts (arity : Arity) (argc : Nat) : Bool :=
  arity.min <= argc && argc <= arity.max

structure FunctionMeta where
  functionId : String
  arity : Arity
  determinism : DeterminismClass
  volatility : VolatilityClass
  hostInteraction : HostInteractionClass
  threadSafety : ThreadSafetyClass
  fecDependencyProfile : FecDependencyProfile
  deriving DecidableEq, Repr

def admitArity (arity : Arity) (args : Args) : Except EvalError Unit :=
  if arity.accepts args.length then
    Except.ok ()
  else
    Except.error (EvalError.arityMismatch arity.min args.length)

theorem admitArity_exact_zero_ok :
    admitArity (Arity.exact 0) [] = Except.ok () := by
  simp [admitArity, Arity.exact, Arity.accepts]

end OxFunc
