import OxFunc.ValueUniverse

namespace OxFunc

-- W111 executable arithmetic substrate for the locale-independent decimal
-- lexeme conversion. Rust uses generated power pairs; this model computes
-- the same mathematical powers directly. Neither embeds oracle outcomes.
structure DecimalDyadic where
  significand : Nat
  exponent : Int
  deriving DecidableEq, Repr

def decimalNearestEven (numerator denominator : Nat) : Nat :=
  let quotient := numerator / denominator
  let remainder := numerator % denominator
  quotient + if 2 * remainder > denominator ||
    (2 * remainder = denominator && quotient % 2 = 1) then 1 else 0

def decimalNarrowDyadic (value : DecimalDyadic) (precision : Nat) : DecimalDyadic :=
  if value.significand = 0 then value else
  let shift := (value.significand.log2 + 1) - precision
  let significand := decimalNearestEven value.significand (2 ^ shift)
  if significand = 2 ^ precision then
    ⟨significand / 2, value.exponent + Int.ofNat shift + 1⟩
  else ⟨significand, value.exponent + Int.ofNat shift⟩

def decimalPower64 (power : Int) : DecimalDyadic :=
  let numerator := if power < 0 then 1 else 10 ^ power.natAbs
  let denominator := if power < 0 then 10 ^ power.natAbs else 1
  let estimate := Int.ofNat numerator.log2 - Int.ofNat denominator.log2
  let below := if estimate < 0 then numerator * 2 ^ estimate.natAbs < denominator
    else numerator < denominator * 2 ^ estimate.natAbs
  let exponent := (if below then estimate - 1 else estimate) - 63
  let scaledNumerator := if exponent < 0 then numerator * 2 ^ exponent.natAbs else numerator
  let scaledDenominator := if exponent < 0 then denominator else denominator * 2 ^ exponent.natAbs
  let significand := decimalNearestEven scaledNumerator scaledDenominator
  if significand = 2 ^ 64 then ⟨significand / 2, exponent + 1⟩
  else ⟨significand, exponent⟩

def decimalMultiply64 (left right : DecimalDyadic) : DecimalDyadic :=
  decimalNarrowDyadic ⟨left.significand * right.significand, left.exponent + right.exponent⟩ 64

def decimalScaleStages (significand : Nat) (scale : Int) : DecimalDyadic :=
  let magnitude := scale.natAbs
  let powers := [magnitude % 16, (magnitude / 16 % 16) * 16, (magnitude / 256) * 256]
  powers.foldl (fun value power => if power = 0 then value else
    decimalMultiply64 value (decimalPower64 (if scale < 0 then -(Int.ofNat power) else Int.ofNat power)))
    ⟨significand, 0⟩

def decimalTextBinary64Bits (significand : Nat) (scale : Int) : Option UInt64 :=
  if significand = 0 then some 0 else
  let value := decimalNarrowDyadic (decimalScaleStages significand scale) 53
  let shift := 53 - (value.significand.log2 + 1)
  let mantissa := value.significand * 2 ^ shift
  let exponent := value.exponent - Int.ofNat shift + 52
  if exponent < -1022 then some 0
  else if exponent > 1023 then none
  else some (UInt64.ofNat ((exponent + 1023).toNat * 2 ^ 52 + mantissa % 2 ^ 52))

theorem decimalTextBinary64Bits_retained_discriminators :
    decimalTextBinary64Bits 935037643088324 (-142) = some 0x25903414b3117a7e
    ∧ decimalTextBinary64Bits 428931203794129 278 = some 0x7cb13168e735f178
    ∧ decimalTextBinary64Bits 484211511671157 283 = some 0x7dbd9d92c8510faf
    ∧ decimalTextBinary64Bits 1 (-308) = some 0
    ∧ decimalTextBinary64Bits 125 (-2) = some 0x3ff4000000000000 := by
  native_decide

end OxFunc
