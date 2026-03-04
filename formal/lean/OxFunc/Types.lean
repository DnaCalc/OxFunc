namespace OxFunc

inductive EvalError where
  | arityMismatch (expected : Nat) (actual : Nat)
  deriving DecidableEq, Repr

inductive Value where
  | number (n : Rat)
  | err (e : EvalError)
  deriving DecidableEq, Repr

abbrev Args := List Value

end OxFunc

