import OxFunc.CoercionPrimitives

namespace OxFunc

private instance gcdLcmExceptDecidable [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b => if h : a = b then isTrue (by cases h; rfl)
      else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b => if h : a = b then isTrue (by cases h; rfl)
      else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

inductive GcdLcmPreparedArg where
  | scalar (value : CoercionInput)
  | rectangular (rows : List (List CoercionInput))
  deriving Repr, DecidableEq

def gcdLcmLimit : Nat := 9007199254740992

def gcdLcmScalar (arrayCell : Bool) (value : CoercionInput) :
    Except WorksheetErrorCode (List Rat) :=
  match value with
  | .emptyCell => .ok (if arrayCell then [0] else [])
  | .logical _ => .error .value
  | _ => match coerceToNumber value with
      | .ok number => .ok [number]
      | .error (.worksheetError code) => .error code
      | .error _ => .error .value

def gcdLcmCells : List CoercionInput → Except WorksheetErrorCode (List Rat)
  | [] => .ok []
  | value :: rest => do
      let here ← gcdLcmScalar true value
      let tail ← gcdLcmCells rest
      pure (here ++ tail)

def gcdLcmCollect : Bool → List GcdLcmPreparedArg →
    Except WorksheetErrorCode (List (List Rat))
  | _, [] => .ok []
  | first, .scalar .missingArg :: rest =>
      if first then .error .na else gcdLcmCollect false rest
  | _, .scalar value :: rest => do
      let here ← gcdLcmScalar false value
      let tail ← gcdLcmCollect false rest
      pure (here :: tail)
  | _, .rectangular [[value]] :: rest => do
      let here ← gcdLcmScalar false value
      let tail ← gcdLcmCollect false rest
      pure (here :: tail)
  | _, .rectangular rows :: rest => do
      let here ← gcdLcmCells (rows.flatMap id)
      let tail ← gcdLcmCollect false rest
      pure (here :: tail)

def gcdLcmPrepare (args : List GcdLcmPreparedArg) :
    Except WorksheetErrorCode (List (List Nat)) := do
  if args.isEmpty || args.length > 255 then throw .value
  let groups ← gcdLcmCollect true args
  let numbers := groups.flatMap id
  if numbers.isEmpty then throw .value
  if numbers.any (fun n => n < 0 || n > (gcdLcmLimit : Rat)) then throw .num
  pure (groups.map (fun group => group.map (fun n => n.num.toNat / n.den)))

def gcdPreparedReduction (args : List GcdLcmPreparedArg) :
    Except WorksheetErrorCode Nat := do
  let groups ← gcdLcmPrepare args
  pure ((groups.flatMap id).foldl Nat.gcd 0)

-- An admitted LCM product is integral. At the top boundary binary64
-- nearest-even maps 2^53+1 to 2^53; every larger integer rounds above it.
def lcmRoundedStep (a b : Nat) : Except WorksheetErrorCode Nat :=
  if a = 0 || b = 0 then .ok 0 else
    let product := (a / Nat.gcd a b) * b
    if product > gcdLcmLimit + 1 then .error .num
    else .ok (min product gcdLcmLimit)

def lcmPreparedReduction (args : List GcdLcmPreparedArg) :
    Except WorksheetErrorCode Nat := do
  let groups ← gcdLcmPrepare args
  let numbers := groups.flatMap id
  if numbers.contains 0 then pure 0
  else numbers.reverse.foldlM lcmRoundedStep 1

theorem lcm_reversed_rounded_reduction :
    lcmPreparedReduction [.rectangular [[.number 3, .number 3, .number 3002399751580331]]] = .error .num
    ∧ lcmPreparedReduction [.scalar (.number 3002399751580331), .scalar (.number 3), .scalar (.number 3)] = .ok 9007199254740992
    ∧ lcmPreparedReduction [.scalar (.number 0), .scalar (.number 9007199254740991), .scalar (.number 9007199254740990)] = .ok 0 := by
  native_decide

theorem gcdLcm_coercion_precedes_domain :
    gcdPreparedReduction [.scalar (.number (-1)), .scalar (.error .div0)] = .error .div0
    ∧ gcdPreparedReduction [.scalar (.text "x"), .scalar (.error .div0)] = .error .value := by
  native_decide

theorem gcdLcm_missing_and_blank_shapes :
    gcdPreparedReduction [.scalar .missingArg, .scalar (.number 6)] = .error .na
    ∧ gcdPreparedReduction [.scalar (.number 6), .scalar .missingArg] = .ok 6
    ∧ gcdPreparedReduction [.scalar .emptyCell] = .error .value
    ∧ gcdPreparedReduction [.rectangular [[.emptyCell, .emptyCell]]] = .ok 0 := by
  native_decide

theorem gcdLcm_inclusive_domain_and_product_rounding :
    gcdPreparedReduction [.scalar (.number 9007199254740992), .scalar (.number 1)] = .ok 1
    ∧ gcdPreparedReduction [.scalar (.number 9007199254740994), .scalar (.number 1)] = .error .num
    ∧ lcmRoundedStep 3 3002399751580331 = .ok 9007199254740992 := by
  native_decide

end OxFunc
