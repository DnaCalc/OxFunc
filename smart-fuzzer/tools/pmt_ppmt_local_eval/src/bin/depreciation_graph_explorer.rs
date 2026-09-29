//! Offline candidate graphs over already-captured, exact-hex public Excel observations.
use oxfunc_core::excel_numeric::research as r;
use serde_json::{json, Value};
fn decode(s: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap())
}
fn xdiv(a: f64, b: f64) -> f64 {
    r::ext_to_f64(
        &r::ext_div(&r::ext_from_f64(a), &r::ext_from_f64(b), r::CW_PC64_RN),
        r::CW_PC64_RN,
    )
}
fn xsub(a: f64, b: f64) -> f64 {
    r::ext_to_f64(
        &r::ext_sub(&r::ext_from_f64(a), &r::ext_from_f64(b), r::CW_PC64_RN),
        r::CW_PC64_RN,
    )
}
fn xadd(a:f64,b:f64)->f64 {
    r::ext_to_f64(&r::ext_add(&r::ext_from_f64(a),&r::ext_from_f64(b),r::CW_PC64_RN),r::CW_PC64_RN)
}
fn pow(a: f64, b: f64, mode: usize) -> f64 {
    if b == 0.0 {
        1.0
    } else if a == 0.0 {
        0.0
    } else {
        match mode {
            0 => a.powf(b),
            1 => r::excel_pow_chain(a, b),
            2 => r::excel_pow_x87_direct(a, b),
            _ => oxfunc_core::functions::power_fn::power_kernel(a, b).unwrap_or(f64::NAN),
        }
    }
}
fn candidate(f: &str, a: &[f64], mode: usize) -> Result<f64, &'static str> {
    if f == "SYD" {return syd_trial(a,mode);}
    if f == "VDB" {
        return vdb(a, mode);
    }
    let (c, s, l, p) = (a[0], a[1], a[2], a[3]);
    let fifth = a
        .get(4)
        .copied()
        .unwrap_or(if f == "DB" { 12.0 } else { 2.0 });
    if c < 0.0 || s < 0.0 || l <= 0.0 || p <= 0.0 {
        return Err("Num");
    }
    if f == "DB" {
        let m = fifth.trunc();
        if m <= 0.0 || m > 12.0 || p.floor() > l + if m < 12.0 { 1.0 } else { 0.0 } {
            return Err("Num");
        }
        if c == 0.0 {
            return Ok(0.0);
        }
        let quotient = s / c;
        let ratio = if !quotient.is_finite() || quotient.abs() < f64::MIN_POSITIVE {
            0.
        } else {
            quotient
        };
        let exponent = 1.0 / l;
        let value = match mode % 4 {
            0 => pow(ratio, exponent, 3),
            1 => r::excel_exp(r::excel_ln(ratio) / l),
            2 => r::excel_exp(xdiv(r::excel_ln(ratio), l)),
            _ => r::excel_pow_x87_direct(ratio, exponent),
        };
        let value = if value.is_finite() && value.abs() >= f64::MIN_POSITIVE {
            value
        } else {
            0.
        };
        let unrounded = 1.0 - value;
        let rate = if mode >= 640 {
            oxfunc_core::functions::round_fn::round_kernel(unrounded, 3)
        } else {
            ((unrounded.abs() * 1000.0 + 0.5).floor() / 1000.0).copysign(unrounded)
        };
        let mut dep = match (mode / 4) % 4 {
            0 => c * rate * m / 12.0,
            1 => c * rate * (m / 12.0),
            2 => {
                if m == 12.0 {
                    c * rate
                } else {
                    c * rate * m / 12.0
                }
            }
            _ => r::x87_mul(r::x87_mul(c, rate), xdiv(m, 12.0)),
        };
        let mut book = c - dep;
        for _t in 2..=(p.floor().min(l.floor())) as usize {
            dep = if (mode / 16) % 2 == 0 {
                book * rate
            } else {
                r::x87_mul(book, rate)
            };

            book = if (mode / 32) % 2 == 0 {
                book - dep
            } else {
                xsub(book, dep)
            };
        }
        if m < 12.0 && p.floor() == l.floor() + 1.0 {
            dep = match (mode / 64) % 10 {
                0 => book * rate * ((12.0 - m) / 12.0),
                1 => book * rate * (1.0 - m / 12.0),
                2 => book * rate * (12.0 - m) / 12.0,
                3 => xdiv(book * rate * (12.0 - m), 12.0),
                4 => book * (rate * (12.0 - m) / 12.0),
                5 => book * xdiv(rate * (12.0 - m), 12.0),
                6 => book * rate / 12.0 * (12.0 - m),
                7 => xdiv(book * rate, 12.0) * (12.0 - m),
                8 => book / 12.0 * rate * (12.0 - m),
                _ => xdiv(book, 12.0) * rate * (12.0 - m),
            };
        }
        return Ok(if dep == 0.0 { 0.0 } else { dep });
    }
    if f == "DDB" {
        if p > l || fifth <= 0.0 {
            return Err("Num");
        }
        if c <= s {
            return Ok(0.0);
        }
        let rate = if (mode / 4) % 2 == 0 {
            fifth / l
        } else {
            xdiv(fifth, l)
        };
        if rate >= 1.0 {
            return Ok(if p <= 1.0 { c - s } else { 0.0 });
        }
        let base = 1.0 - rate;
        let factor = pow(base, (p - 1.0).max(0.0), mode % 4);
        let book = if (mode / 8) % 2 == 0 {
            c * factor
        } else {
            r::x87_mul(c, factor)
        };
        let dep = if (mode / 16) % 2 == 0 {
            book * rate
        } else {
            r::x87_mul(book, rate)
        };
        return Ok(dep.min(book - s).max(0.0));
    }
    Err("Num")
}

fn syd_trial(a:&[f64],mode:usize)->Result<f64,&'static str>{
    let (c,s,l,p)=(a[0],a[1],a[2],a[3]);
    if s<0. || l<=0. || p<=0. || (p>l && p-l>=f64::MIN_POSITIVE) {return Err("Num");}
    let sub=|a,b|if mode&64==0 {a-b}else{xsub(a,b)};
    let mul=|a,b|if mode&128==0 {a*b}else{r::x87_mul(a,b)};
    let div=|a,b|if mode&256==0 {a/b}else{xdiv(a,b)};
    let life_plus_one=if mode&2048!=0 {xadd(l,1.)}else{l+1.};
    let rem=match mode%4 {0=>sub(l,p)+1.,1=>sub(life_plus_one,p),_=>sub(l,sub(p,1.))};
    let den=match (mode/4)%4 {0=>mul(l,life_plus_one)/2.,1=>mul(l/2.,life_plus_one),2=>mul(l,life_plus_one/2.),_=>mul(l,life_plus_one)};
    let den=if !den.is_finite() || den.abs()<f64::MIN_POSITIVE {0.}else{den};
    if den==0. {return Err("Div0");}
    let publish=|n:f64| if n.abs()<f64::MIN_POSITIVE || !n.is_finite() {0.}else{n};
    let basis=if mode&512!=0 {publish(sub(c,s))}else{sub(c,s)};
    let num=if mode&1024!=0 {publish(mul(basis,rem))}else{mul(basis,rem)};
    let result=match (mode/16)%4 {0=>div(num,den),1=>mul(div(basis,den),rem),2=>mul(basis,div(rem,den)),_=>div(num*2.,den)};
    if !result.is_finite() {Err("Num")}else{Ok(if result.abs()<f64::MIN_POSITIVE {0.}else{result})}
}
fn raw_ddb(c: f64, s: f64, l: f64, p: f64, f: f64) -> Result<f64, &'static str> {
    let rate = xdiv(f, l);
    let rate = if rate.abs() < f64::MIN_POSITIVE {
        0.
    } else {
        rate
    };
    let power = if p <= 1. {
        1.
    } else {
        pow((1. - rate).max(0.), p - 1., 3)
    };
    let book = r::x87_mul(c, power);
    let dep = r::x87_mul(book, rate);
    if !dep.is_finite() {
        return Err("Num");
    }
    let result = dep.min(book - s).max(0.);
    Ok(if result.abs() < f64::MIN_POSITIVE {
        0.
    } else {
        result
    })
}
fn vdb_switch_trial(c:f64,s:f64,l:f64,start:f64,end:f64,factor:f64,mode:usize)->Result<f64,&'static str>{
    let phase=|cost:f64,life:f64,duration:f64|->Result<(f64,Option<f64>),&'static str>{
        let mut total=0.;let mut book=cost;let mut switched=None;
        for year in 0..duration.ceil() as usize {
            let remaining=if mode&1==0 {cost-s-total}else{book-s};
            let raw=raw_ddb(cost,0.,l,(year+1) as f64,factor)?;
            let dd=if mode&2==0 {raw.min(cost-s)}else{raw.min(remaining)};
            let sl=xdiv(remaining,life-year as f64);
            let amount=if let Some(sl)=switched {sl}else if sl>dd {switched=Some(sl);sl}else{dd};
            let take=(duration-year as f64).min(1.);
            total+=amount*take;
            book-=amount*take;
        }
        if mode&4!=0 && cost>=s {total=total.min(cost-s);}
        Ok((total,switched))
    };
    let (advance,switched)=phase(c,l,start)?;
    if mode&8==0 {if let Some(sl)=switched {return Ok(sl*(end-start));}}
    let rem_life=if mode&16==0 && factor>1. {(l-start).max(1.).min(l)}else{l-start};
    phase(c-advance,rem_life,end-start).map(|x|x.0)
}

fn vdb_shift_trial(c:f64,s:f64,l:f64,start:f64,end:f64,factor:f64,mode:usize)->Result<f64,&'static str>{
    if c<s && mode&2199023255552==0 {
        let mut book=c;
        // After the second subtraction book is exactly salvage by Sterbenz;
        // retain the first subtraction's residual instead of replacing it by 0.
        for _ in 0..(start.floor() as usize).min(2) {book-=book-s;}
        return Ok((book-s)*(end-start).min(1.));
    }
    let stage=|value:f64|if mode&281474976710656!=0 && value.abs()<f64::MIN_POSITIVE {0.}else{value};
    let greater=|a:f64,b:f64|if mode&(1usize<<56)!=0 {a>b && a-b>=f64::MIN_POSITIVE/16.}else if mode&(1usize<<55)!=0 {a>b && a-b>=f64::MIN_POSITIVE}else{a>b};
    let rate=stage(if mode&(1usize<<58)!=0 {factor/l}else{xdiv(factor,l)});
    let declining=|book:f64|stage(match (mode/256)%4 {0=>if mode&(1usize<<57)!=0 {book*rate}else{r::x87_mul(book,rate)},1=>xdiv(book,l)*factor,2=>xdiv(book*factor,l),_=>book*factor/l});
    let dd_first=declining(c);
    let sl_calc=|cost:f64,salvage:f64,life:f64|stage(if mode&65536!=0 {r::ext_to_f64(&r::ext_div(&r::ext_sub(&r::ext_from_f64(cost),&r::ext_from_f64(salvage),r::CW_PC64_RN),&r::ext_from_f64(life),r::CW_PC64_RN),r::CW_PC64_RN)}else{xdiv(cost-salvage,life)});
    let sl_first=sl_calc(c,s,l);
    if mode&140737488355328!=0 && (!rate.is_finite() || !dd_first.is_finite() || !sl_first.is_finite()) {return Err("Num");}
    if c>=s && greater(sl_first,dd_first.min(c-s)) {
        if mode&70368744177664!=0 {
            let sl=r::ext_div(&r::ext_from_f64(c-s),&r::ext_from_f64(l),r::CW_PC64_RN);
            return Ok(r::ext_to_f64(&r::ext_mul(&sl,&r::ext_from_f64(end-start),r::CW_PC64_RN),r::CW_PC64_RN));
        }
        let result=if mode&8796093022208!=0 {r::x87_mul(sl_first,end-start)}else{sl_first*(end-start)};
        return Ok(if mode&(1usize<<54)!=0 {stage(result)}else{result});
    }
    let fraction=start-start.floor();
    let first=dd_first.min(c-s).max(0.);
    let initial_product=if mode&128!=0 {r::x87_mul(first,fraction)}else{first*fraction};
    let first_part=if mode&(1usize<<51)!=0 {initial_product}else{stage(initial_product)};
    let mut book=if mode&64!=0 || mode&4398046511104!=0 {xsub(c,first_part)}else{c-first_part};
    if mode&262144!=0 {book=r::ext_to_f64(&r::ext_sub(&r::ext_from_f64(c),&r::ext_mul(&r::ext_from_f64(first),&r::ext_from_f64(fraction),r::CW_PC64_RN),r::CW_PC64_RN),r::CW_PC64_RN);}
    let mut basis=if mode&32768!=0 {(c-s)-first_part}else{book-s};
    let mut remaining_life=l-fraction;
    let mut absolute_elapsed=fraction;
    let mut sl_fixed=None;
    let starting_book=book;let mut advance_total=0.;let mut invocation=0;
    let mut step=|duration:f64,new_life:Option<f64>|->Result<f64,&'static str>{
        // If advance already entered straight-line mode, use the original
        // interval subtraction instead of the shifted endpoint storage path.
        let duration=if invocation==1 && mode&8589934592!=0 && sl_fixed.is_some() {end-start}else{duration};
        if invocation==1 && mode&4194304!=0 {book=starting_book-advance_total;}
        if let Some(value)=new_life {remaining_life=value; if mode&32!=0 {sl_fixed=None;}}
        let mut total=0.;let phase_cost=book;let mut ext_total=r::ext_from_f64(0.);let mut overall_total=advance_total;
        let mut interval_left=start;
        let mut year=0usize;
        loop {
            if mode&17592186044416!=0 && invocation==1 && year as f64>=end-start {break;}
            if mode&1099511627776!=0 && invocation==1 {
                if interval_left>=end {break;}
            }else if year as f64>=duration {break;}
            let dd=if mode&2==0 {declining(r::x87_mul(phase_cost,pow((1.-rate).max(0.),year as f64,3)))}else{declining(book)};
            let available=if mode&67108864!=0 {(phase_cost-s)-total}else if mode&16384!=0 {basis}else{book-s};
            let life_left=if mode&137438953472!=0 {l-(if invocation==0 {fraction+year as f64}else{start+year as f64})}else if mode&2097152!=0 {l-absolute_elapsed}else{remaining_life};
            let denominator=if mode&1==0 && dd>available {life_left.max(1.)}else{life_left};
            let sl=if mode&32==0 && sl_fixed.is_some() {sl_fixed.unwrap()}else if mode&16384!=0 {xdiv(available,denominator)}else{sl_calc(book,s,denominator)};
            if mode&140737488355328!=0 && (!dd.is_finite() || !sl.is_finite()) {return Err("Num");}
            if mode&(1usize<<53)!=0 && !sl_calc(book,s,life_left).is_finite() {return Err("Num");}
            let previously_switched=sl_fixed.is_some();
            let amount=if mode&4294967296!=0 && sl_fixed.is_none() && dd>available {available}else if greater(sl,dd.min(available)) {sl_fixed=Some(sl);sl}else{dd.min(available)};
            let take=if mode&68719476736!=0 && invocation==1 {
                let absolute_left=if mode&549755813888!=0 {interval_left}else{start+year as f64};
                let remaining=end-absolute_left;
                if sl_fixed.is_some() {remaining}else if mode&274877906944!=0 {(absolute_left+1.).min(end)-absolute_left}else{remaining.min(1.)}
            }else if mode&17179869184!=0 && invocation==1 && year==0 && sl_fixed.is_some() {end-start}else if mode&128!=0 && sl_fixed.is_some() {duration-year as f64}else{(duration-year as f64).min(1.)};
            let raw_part=if mode&8!=0 {r::x87_mul(amount,take)}else{amount*take};
            let mut part=if mode&(1usize<<52)!=0 || (mode&(1usize<<50)!=0 && take==1.) {raw_part}else{stage(raw_part)};
            if sl_fixed.is_some() && !previously_switched {
                part=match (mode/16777216)%4 {1=>xdiv(available*take,denominator),2=>available*xdiv(take,denominator),3=>xdiv(r::x87_mul(available,take),denominator),_=>part};
            }
            let old_total=total;
            overall_total=xadd(overall_total,part);
            total=if mode&131072!=0 {r::ext_to_f64(&r::ext_add(&r::ext_from_f64(total),&r::ext_mul(&r::ext_from_f64(amount),&r::ext_from_f64(take),r::CW_PC64_RN),r::CW_PC64_RN),r::CW_PC64_RN)}else if mode&16!=0 {xadd(total,part)}else{total+part};
            if mode&268435456!=0 {ext_total=r::ext_add(&ext_total,&r::ext_from_f64(part),r::CW_PC64_RN);total=r::ext_to_f64(&ext_total,r::CW_PC64_RN);}
            if mode&134217728!=0 && sl_fixed.is_none() && take<1. {total=xsub(xadd(old_total,amount),amount*(1.-take));}
            if sl_fixed.is_some() && mode&128!=0 {
                total=match (mode/524288)%4 {
                    1=>xadd(xadd(old_total,amount*take.floor()),amount*(take-take.floor())),
                    2=>xsub(xadd(old_total,amount*take.ceil()),amount*(take.ceil()-take)),
                    3=>xadd(xadd(old_total,amount*(take.ceil()-1.)),amount*(1.-(take.ceil()-take))),
                    _=>total,
                };
            }
            book=if (invocation==0 && mode&536870912!=0)||(invocation==1 && mode&1073741824!=0) {xsub(book,part)}else if mode&64!=0 {xsub(phase_cost,total)}else if mode&4!=0 {xsub(book,part)}else{book-part};
            basis=if mode&4!=0 {xsub(basis,part)}else{basis-part};
            remaining_life-=take;
            absolute_elapsed+=take;
            if mode&35184372088832!=0 && invocation==1 && end-interval_left<=1. {break;}
            interval_left+=1.;
            if mode&128!=0 && sl_fixed.is_some() {break;}
            year+=1;
        }
        if invocation==0 {advance_total=total;}else if mode&2147483648!=0 {total=overall_total-advance_total;}else if mode&8388608!=0 {total=(advance_total+total)-advance_total;}
        invocation+=1;
        Ok(total)
    };
    step(start.floor(),None)?;
    let shifted_end=if mode&34359738368!=0 {xsub(end,fraction)}else{end-fraction};
    let duration=match (mode/1024)%4 {0=>end-start,1=>(end-start.floor())-fraction,2=>shifted_end-start.floor(),_=>(end-start.floor())-(start-start.floor())};
    let reset_life=match (mode/4096)%4 {0=>if mode&256!=0 {Some(l-start)}else{None},1=>Some(l-start),2=>Some((l-fraction)-start.floor()),_=>Some((l-start.floor())-fraction)};
    step(duration,reset_life)
}

fn vdb(a: &[f64], mode: usize) -> Result<f64, &'static str> {
    let (c, s, l, start, end) = (a[0], a[1], a[2], a[3], a[4]);
    let factor = a.get(5).copied().unwrap_or(2.);
    let ns = a.get(6).copied().unwrap_or(0.) != 0.;
    if c < 0. || s < 0. || l <= 0. || factor < 0. || start < 0. || end < start || end > l {
        return Err("Num");
    }
    if start == end {
        return Ok(0.);
    }
    let rate = if mode & 1 == 0 {
        factor / l
    } else {
        xdiv(factor, l)
    };
    if ns {
        let mut total = 0.;
        let first = start.floor() as usize;
        let last = end.ceil() as usize;
        for p in first..last {
            let dep = raw_ddb(c, s, l, (p + 1) as f64, factor)?;
            let left = (start - p as f64).max(0.);
            let right = ((p + 1) as f64 - end).max(0.);
            let mul = |a: f64, b: f64| {
                if mode & 4 == 0 {
                    a * b
                } else {
                    r::x87_mul(a, b)
                }
            };
            let sub = |a: f64, b: f64| if mode & 8 == 0 { a - b } else { xsub(a, b) };
            let contribution = match mode % 4 {
                0 => sub(mul(dep, 1. - right), mul(dep, left)),
                1 => (dep - dep * right) - dep * left,
                2 => (dep - dep * left) - dep * right,
                _ => dep,
            };
            total = if mode & 16 == 0 {
                total + contribution
            } else {
                r::ext_to_f64(
                    &r::ext_add(
                        &r::ext_from_f64(total),
                        &r::ext_from_f64(contribution),
                        r::CW_PC64_RN,
                    ),
                    r::CW_PC64_RN,
                )
            };
        }
        if mode % 4 == 3 {
            let last_dep = raw_ddb(c, s, l, last as f64, factor)?;
            total -= last_dep * (last as f64 - end);
            let first_dep = raw_ddb(c, s, l, (first + 1) as f64, factor)?;
            total -= first_dep * (start - first as f64);
        }
        return Ok(if total.abs() < f64::MIN_POSITIVE {
            0.
        } else {
            total
        });
    }

    if mode>=64 {
        let value=vdb_shift_trial(c,s,l,start,end,factor,mode-64)?;
        if mode&140737488355328!=0 {
            if !value.is_finite() {return Err("Num");}
            if value==0. || (mode&(1usize<<49)==0 && value.abs()<f64::MIN_POSITIVE) {return Ok(0.);}
        }
        return Ok(value);
    }
    if mode>=32 {return vdb_switch_trial(c,s,l,start,end,factor,mode-32);}
    let phase = |c: f64, life: f64, duration: f64| {
        let mut total = 0.;
        let mut book = c;
        let mut fixed_sl = None;
        for p in 0..duration.ceil() as usize {
            let remaining = book - s;
            let dd = match (mode / 2) % 3 {
                0 => match mode / 256 {
                    0 => book * rate,
                    1 => book * factor / l,
                    2 => xdiv(book * factor, l),
                    3 => book / l * factor,
                    4 => xdiv(r::x87_mul(book, factor), l),
                    _ => r::ext_to_f64(
                        &r::ext_div(
                            &r::ext_mul(
                                &r::ext_from_f64(book),
                                &r::ext_from_f64(factor),
                                r::CW_PC64_RN,
                            ),
                            &r::ext_from_f64(l),
                            r::CW_PC64_RN,
                        ),
                        r::CW_PC64_RN,
                    ),
                },
                1 => c * pow(1. - rate, p as f64, 3) * rate,
                _ => oxfunc_core::functions::depreciation_family::ddb_kernel(
                    c,
                    s,
                    l,
                    (p + 1) as f64,
                    factor,
                )
                .unwrap_or(0.),
            };
            let sl = if mode & 8 == 0 {
                remaining / (life - p as f64)
            } else {
                xdiv(remaining, life - p as f64)
            };
            let sl = if mode & 16 != 0 {
                fixed_sl.unwrap_or(sl)
            } else {
                sl
            };
            let comparison = if mode & 32 != 0 {
                dd.min(remaining)
            } else {
                dd
            };
            let dep = if sl > comparison {
                fixed_sl = Some(sl);
                sl
            } else {
                dd.min(remaining)
            };
            let take = (duration - p as f64).min(1.);
            total += dep * take;
            book = if mode & 64 != 0 {
                c - total
            } else {
                book - dep * take
            };
        }
        (total, book)
    };
    let (advance, book) = phase(c, l, start);
    let book = if mode & 128 != 0 { c - advance } else { book };
    let answer = phase(book, l - start, end - start).0;
    Ok(if answer == 0. { 0. } else { answer })
}
fn search_db_stores() {
    let mut state=0x202609291650abefu64;
    let mut next = || { state ^= state << 13; state ^= state >> 7; state ^= state << 17; state };
    let mut counts=[0usize;5];
    let mut probes=Vec::new();
    for trial in 0..4_000_000usize {
        if counts.iter().all(|n|*n>=100) {break;}
        let cost=f64::from_bits((((next()%1800)+124)<<52)|(next()&0x000fffffffffffff));
        let rate=((next()%9999) as f64-4999.)/1000.;
        if rate==0. || rate>=1. {continue;}
        let month=((next()%11)+1) as f64;
        let first=cost*rate*(month/12.);
        let book=xsub(cost,first);
        let quotient=book/12.;
        let product=quotient*rate;
        let pairs=[
            (cost*rate,r::x87_mul(cost,rate)),
            ((cost*rate)*(month/12.),r::x87_mul(cost*rate,month/12.)),
            (quotient,xdiv(book,12.)),
            (product,r::x87_mul(quotient,rate)),
            (product*(12.-month),r::x87_mul(product,12.-month)),
        ];
        for site in 0..5 {
            if counts[site]>=100 || pairs[site].0.to_bits()==pairs[site].1.to_bits() {continue;}
            let life=if site<2 {2.}else{1.};
            let period=if site<2 {1.}else{2.};
            let salvage=cost*(1.-rate).powf(life);
            if !salvage.is_finite() || salvage.abs()<f64::MIN_POSITIVE {continue;}
            let actual_rate=oxfunc_core::functions::round_fn::round_kernel(1.-pow(xdiv(salvage,cost),xdiv(1.,life),3),3);
            if actual_rate.to_bits()!=rate.to_bits() {continue;}
            let args=[cost,salvage,life,period,month].map(|x|format!("0x{:016x}",x.to_bits()));
            probes.push(json!({"probe":{"id":format!("w111db-store-{site}-{:04}",counts[site]),"args":args},"probe_region":format!("selected-operation-store-site-{site}"),"search_trial":trial}));
            counts[site]+=1;
        }
    }
    println!("{}",json!({"function":"DB","probes":probes,"search_counts":counts,"sites":["first-cost-rate","first-fraction-product","final-book-division","final-rate-product","final-month-product"]}));
}

fn ddb_power_stores(path: &str) {
    let v: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    for mode in 0..8 {
        let mut misses=Vec::new(); let mut exact=0;
        for w in v["witnesses"].as_array().unwrap() {
            let a: Vec<f64> = w["args"].as_array().unwrap().iter().map(|x|decode(x.as_str().unwrap())).collect();
            let rate=xdiv(a[4],a[2]);
            let base=if mode&4!=0 {xsub(1.,rate)} else {1.-rate};
            let exp=a[3]-1.;
            let power=if exp.fract()==0. && exp<(u32::MAX as f64) {
                let mut n=exp as u32; let mut b=base; let mut acc=1.;
                while n>0 {if n&1!=0 {acc=if mode&1!=0 {r::x87_mul(acc,b)} else {acc*b};} n>>=1; if n>0 {b=if mode&2!=0 {r::x87_mul(b,b)} else {b*b};}}
                acc
            } else {r::excel_pow_chain(base,exp)};
            let book=r::x87_mul(a[0],power);let dep=r::x87_mul(book,rate).min(book-a[1]).max(0.);
            let actual=format!("0x{:016x}",dep.to_bits());
            if actual==w["expected_bits"].as_str().unwrap() {exact+=1;} else if misses.len()<12 {misses.push(json!({"id":w["id"],"args":a,"actual":actual,"expected":w["expected_bits"]}));}
        }
        println!("{}",json!({"mode":mode,"exact":exact,"misses":misses}));
    }
}

fn main() {
    if std::env::args().nth(1).as_deref()==Some("--search-syd-stores") {
        let mut state=0x202609292025fabcu64;let mut probes=Vec::new();
        let mut random=||{state^=state<<13;state^=state>>7;state^=state<<17;state};
        for _ in 0..1_000_000 {
            let c=f64::from_bits(((900+random()%250)<<52)|(random()&0xfffffffffffff));
            let s=c*(random()%100000) as f64/100000.;
            let l=f64::from_bits(((1018+random()%15)<<52)|(random()&0xfffffffffffff));
            let p=l*(1+random()%99999) as f64/100000.;let a=[c,s,l,p];
            let base=syd_trial(&a,513).unwrap();
            let distinguished:Vec<_>=[577,641,769,897].into_iter().filter(|m|syd_trial(&a,*m).unwrap().to_bits()!=base.to_bits()).collect();
            if !distinguished.is_empty() {probes.push(json!({"probe":{"id":format!("w111syd-stores-{:04}",probes.len()),"args":a.map(|x|format!("0x{:016x}",x.to_bits()))},"probe_region":format!("ordinary-versus-extended-stores-{distinguished:?}")}));}
            if probes.len()>=300 {break;}
        }
        println!("{}",json!({"function":"SYD","probes":probes}));return;
    }
    if std::env::args().nth(1).as_deref()==Some("--syd-one") {
        let mode=std::env::args().nth(2).unwrap().parse::<usize>().unwrap();
        let path=std::env::args().nth(3).unwrap();
        let text=std::fs::read_to_string(path).unwrap();
        let doc:Value=serde_json::from_str(text.trim_start_matches('\u{feff}')).unwrap();
        let mut exact=0;let mut misses=Vec::new();
        for w in doc["witnesses"].as_array().unwrap() {
            let a:Vec<f64>=w["args"].as_array().unwrap().iter().map(|x|decode(x.as_str().unwrap())).collect();
            let actual=match syd_trial(&a,mode) {Ok(v)=>format!("0x{:016x}",v.to_bits()),Err(e)=>format!("error:{e}")};
            if actual==w["expected_bits"].as_str().unwrap() {exact+=1;}else{misses.push(json!({"id":w["id"],"args":a,"actual":actual,"expected":w["expected_bits"]}));}
        }
        println!("{}",json!({"mode":mode,"exact":exact,"misses":misses}));return;
    }
    if std::env::args().nth(1).as_deref()==Some("--vdb-partial-stores") {
        let path=std::env::args().nth(2).unwrap();
        let text=std::fs::read_to_string(path).unwrap();
        let doc:Value=serde_json::from_str(text.trim_start_matches('\u{feff}')).unwrap();
        for w in doc["witnesses"].as_array().unwrap() {
            let a:Vec<f64>=w["args"].as_array().unwrap().iter().map(|x|decode(x.as_str().unwrap())).collect();
            assert!(a[3].floor()==0. && a[4]<=1. && a[2]>=1.);
            let dep=oxfunc_core::functions::depreciation_family::ddb_kernel(a[0],a[1],a[2],1.,a[5]).unwrap();
            let fraction=1.-(1.-a[4]);
            let rows:Vec<_>=(0..4).map(|mode| {
                let mul=|x,y|if mode&1==0 {x*y}else{r::x87_mul(x,y)};
                let v=if mode&2==0 {mul(dep,fraction)-mul(dep,a[3])}else{xsub(mul(dep,fraction),mul(dep,a[3]))};
                json!({"mode":mode,"value":format!("0x{:016x}",v.to_bits())})
            }).collect();
            println!("{}",json!({"id":w["id"],"expected":w["expected_bits"],"dep":format!("0x{:016x}",dep.to_bits()),"trials":rows}));
        }
        return;
    }
    if std::env::args().nth(1).as_deref()==Some("--vdb-shift-one") {
        let mode:usize=std::env::args().nth(2).unwrap().parse().unwrap();
        let v:Value=serde_json::from_slice(&std::fs::read(std::env::args().nth(3).unwrap()).unwrap()).unwrap();
        let mut misses=Vec::new();let mut exact=0;
        for w in v["witnesses"].as_array().unwrap() {
            let a:Vec<f64>=w["args"].as_array().unwrap().iter().map(|s|decode(s.as_str().unwrap())).collect();
            assert!(a[4]<1000.,"Research interval bound exceeded");
            let got=vdb(&a,mode);let expected=w["expected_bits"].as_str().unwrap();
            let actual=match got {Ok(x)=>format!("0x{:016x}",x.to_bits()),Err(e)=>format!("error:{e}")};
            if actual==expected {exact+=1;}else{let ulp=got.ok().filter(|_|expected.starts_with("0x")).map(|x|x.to_bits().abs_diff(decode(expected).to_bits()));misses.push(json!({"id":w["id"],"args":a,"actual":actual,"expected":expected,"ulp":ulp}));}
        }
        println!("{}",json!({"mode":mode,"exact":exact,"misses":misses}));return;
    }
    if std::env::args().nth(1).as_deref()==Some("--search-ddb-base") {
        let mut state=0x202609291855abefu64;let mut probes=Vec::new();
        for i in 0..2_000_000u64 {
            state^=state<<13;state^=state>>7;state^=state<<17;
            let life=f64::from_bits((1050u64+(state%20))<<52 | (state&0x000fffffffffffff));
            let factor=[0.5,1.,2.,10.,17.][(i%5) as usize];let rate=xdiv(factor,life);
            if (1.-rate).to_bits()!=xsub(1.,rate).to_bits() {
                for period in [1.25,life*0.75,life] {
                    let a=[1.,0.,life,period,factor].map(|x|format!("0x{:016x}",x.to_bits()));
                    probes.push(json!({"probe":{"id":format!("w111ddb-base-{:04}",probes.len()),"args":a},"probe_region":"selected-ordinary-versus-x87-base-subtraction"}));
                }
                if probes.len()>=300 {break;}
            }
        }
        println!("{}",json!({"function":"DDB","probes":probes}));return;
    }
    if std::env::args().nth(1).as_deref()==Some("--ddb-power-stores") {ddb_power_stores(&std::env::args().nth(2).unwrap());return;}
    if std::env::args().nth(1).as_deref()==Some("--search-db-stores") {search_db_stores();return;}
    for path in std::env::args().skip(1) {
        let v: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let f = v["function"].as_str().unwrap();
        let mut scores = Vec::new();
        for mode in 0..if f == "SYD" {512} else if f == "DB" {
            1280
        } else if f == "VDB" {
            if path.contains("noswitch") {
                32
            } else {
                1088
            }
        } else {
            32
        } {
            let mut exact = 0;
            let mut small = 0;
            let mut misses = Vec::new();
            for w in v["witnesses"].as_array().unwrap() {
                let args: Vec<f64> = w["args"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| decode(x.as_str().unwrap()))
                    .collect();
                let got = candidate(f, &args, mode);
                let want = w["expected_bits"].as_str().unwrap();
                let actual = match got {
                    Ok(x) => format!("0x{:016x}", x.to_bits()),
                    Err(e) => format!("error:{e}"),
                };
                if actual == want {
                    exact += 1;
                } else {
                    if let Ok(x) = got {
                        if want.starts_with("0x")
                            && x.to_bits().abs_diff(decode(want).to_bits()) <= 4
                        {
                            small += 1;
                        }
                    }
                    if misses.len() < 20 {
                        misses.push(
                            json!({"id":w["id"],"args":args,"actual":actual,"expected":want}),
                        );
                    }
                }
            }
            scores.push(json!({"mode":mode,"exact":exact,"small_ulp":small,"misses":misses}));
        }
        scores.sort_by_key(|s| std::cmp::Reverse(s["exact"].as_u64().unwrap()));
        println!(
            "{}",
            json!({"function":f,"path":path,"best":&scores[..3],"selected":scores.iter().filter(|s|[36,37,40,52,64,65,66,67].contains(&s["mode"].as_i64().unwrap())).collect::<Vec<_>>()})
        );
    }
}

