# ERFC campaign REGION_MAP

26-bit PQR+t cubes: bits 0-6 P, 7-13 Q, 14-22 R, 23-25 t-formation spills.
Native / every-op-spill / all-stage-spill are single configs (mask-insensitive).
First cube is R1m/z0/r0 = x87-DR + store uv + mid_cut 1.5 (assoc-race bar axes).

| id | space | next | note |
|---|---|---|---|
| R0 | named F + implied-F | done | baselines |
| R0c | NSWC P/Q/R ±1 ULP | done | last-bit decimals |
| R1base | mask-insensitive arith | done | Native/Every/Pc53/Stage |
| R1m/z0/r0 | 0..0x4000000 | done | NSWC 26-bit PQR+t mask, mid_cut=1.5 AA/BB above, zz_dr=true uvS |
| R1m/z0/r1 | 0..0x4000000 | done | NSWC 26-bit PQR+t mask, mid_cut=1.5 AA/BB above, zz_dr=true uvC |
| R1m/z0/r2 | 0..0x4000000 | done | NSWC 26-bit PQR+t mask, mid_cut=1.5 AA/BB above, zz_dr=true uvR |
| R1m/z1/r0 | 0..0x4000000 | done | NSWC 26-bit PQR+t mask, mid_cut=1.5 AA/BB above, zz_dr=false uvS |
| R1m/z1/r1 | 0..0x4000000 | done | NSWC 26-bit PQR+t mask, mid_cut=1.5 AA/BB above, zz_dr=false uvC |
| R1m/z1/r2 | 0..0x4000000 | done | NSWC 26-bit PQR+t mask, mid_cut=1.5 AA/BB above, zz_dr=false uvR |
| R1/z0/r0 | 0..0x4000000 | done | NSWC 26-bit PQR+t mask, PQR on [0.5,4), zz_dr=true uvS |
| R1/z0/r1 | 0..0x4000000 | 0x000f000/0x4000000 | NSWC 26-bit PQR+t mask, PQR on [0.5,4), zz_dr=true uvC |
| R1/z0/r2 | 0..0x4000000 | 0x0000000/0x4000000 | NSWC 26-bit PQR+t mask, PQR on [0.5,4), zz_dr=true uvR |
| R1/z1/r0 | 0..0x4000000 | 0x0000000/0x4000000 | NSWC 26-bit PQR+t mask, PQR on [0.5,4), zz_dr=false uvS |
| R1/z1/r1 | 0..0x4000000 | 0x0000000/0x4000000 | NSWC 26-bit PQR+t mask, PQR on [0.5,4), zz_dr=false uvC |
| R1/z1/r2 | 0..0x4000000 | 0x0000000/0x4000000 | NSWC 26-bit PQR+t mask, PQR on [0.5,4), zz_dr=false uvR |
| R4/z0/r0 | 0..0x0080000 | 0x0000000/0x0080000 | NSWC AA/BB 19-bit store-mask, mid_cut=1.5, zz_dr=true uvS |
| R4/z0/r1 | 0..0x0080000 | 0x0000000/0x0080000 | NSWC AA/BB 19-bit store-mask, mid_cut=1.5, zz_dr=true uvC |
| R4/z0/r2 | 0..0x0080000 | 0x0000000/0x0080000 | NSWC AA/BB 19-bit store-mask, mid_cut=1.5, zz_dr=true uvR |
| R4/z1/r0 | 0..0x0080000 | 0x0000000/0x0080000 | NSWC AA/BB 19-bit store-mask, mid_cut=1.5, zz_dr=false uvS |
| R4/z1/r1 | 0..0x0080000 | 0x0000000/0x0080000 | NSWC AA/BB 19-bit store-mask, mid_cut=1.5, zz_dr=false uvC |
| R4/z1/r2 | 0..0x0080000 | 0x0000000/0x0080000 | NSWC AA/BB 19-bit store-mask, mid_cut=1.5, zz_dr=false uvR |
| R2/z0/r0 | 0..0x0010000 | 0x0000000/0x0010000 | Cody C/D 16-bit store-mask, zz_dr=true uvS |
| R2/z0/r1 | 0..0x0010000 | 0x0000000/0x0010000 | Cody C/D 16-bit store-mask, zz_dr=true uvC |
| R2/z0/r2 | 0..0x0010000 | 0x0000000/0x0010000 | Cody C/D 16-bit store-mask, zz_dr=true uvR |
| R2/z1/r0 | 0..0x0010000 | 0x0000000/0x0010000 | Cody C/D 16-bit store-mask, zz_dr=false uvS |
| R2/z1/r1 | 0..0x0010000 | 0x0000000/0x0010000 | Cody C/D 16-bit store-mask, zz_dr=false uvC |
| R2/z1/r2 | 0..0x0010000 | 0x0000000/0x0010000 | Cody C/D 16-bit store-mask, zz_dr=false uvR |
| R1p/mid15 | 0..0x4000000 | 0x0000000/0x4000000 | NSWC 26-bit PQR+t, X87Pc53, zz_dr, uv store, mid_cut=1.5 |

best_mid: 3336 / bar 3332  `R1m/z0/r0 /mask=0048000`  max_ulp=7  pins=1
best_all: 6005  `nswc_x87cont_zzdr_store_mid15`
