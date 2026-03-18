namespace OxFunc

inductive CellInfoQuery where
  | address
  | row
  | col
  | contents
  | type
  | filename
  | format
  | color
  | parentheses
  | prefix
  | protect
  | width
  deriving DecidableEq, Repr

inductive InfoQuery where
  | directory
  | numFile
  | origin
  | osVersion
  | recalc
  | release
  | system
  | memAvail
  | memUsed
  | totMem
  deriving DecidableEq, Repr

def providerCanServeCellQuery (_q : CellInfoQuery) : Bool :=
  true

def isExplicitReferenceHostQuery (q : CellInfoQuery) : Bool :=
  match q with
  | .filename | .format | .color | .parentheses | .prefix | .protect | .width => true
  | .address | .row | .col | .contents | .type => false

theorem allCellInfoQueries_are_provider_addressable (q : CellInfoQuery) :
    providerCanServeCellQuery q = true := by
  cases q <;> rfl

theorem explicitReferenceHostQueries_are_exactly_host_sensitive (q : CellInfoQuery) :
    isExplicitReferenceHostQuery q = true ↔
      q = .filename ∨ q = .format ∨ q = .color ∨ q = .parentheses
        ∨ q = .prefix ∨ q = .protect ∨ q = .width := by
  cases q <;> simp [isExplicitReferenceHostQuery]

end OxFunc
