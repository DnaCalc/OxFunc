"""General integer-domain hypotheses for the GCD/LCM black-box captures."""
import json,math,struct,sys
from collections import Counter
from pathlib import Path

def decode(bits):return struct.unpack('>d',bytes.fromhex(bits[2:]))[0]
def encode(n):return '0x'+struct.pack('>d',float(n)).hex()
def main():
    root=Path(sys.argv[1]);report={}
    for fn in ['gcd','lcm']:
        source=root/'answers'/f'answers-{fn}.json'
        if not source.exists():source=root/f'answers-{fn}.json'
        rows=json.loads(source.read_text(encoding='utf-8'))['witnesses'];counts=Counter();misses={}
        for row in rows:
            args=[decode(x) for x in row['args']]
            for input_strict in [False,True]:
                for result_strict in [False,True]:
                    for rounded_result in [False,True]:
                        name=f'input_{"ge" if input_strict else "gt"}_result_{"ge" if result_strict else "gt"}_{"float" if rounded_result else "exact"}'
                        if any(n<0 or (n>=2**53 if input_strict else n>2**53) for n in args):answer='error:Num'
                        else:
                            ints=list(map(int,args));result=math.gcd(*ints) if fn=='gcd' else math.lcm(*ints)
                            check=float(result) if rounded_result else result
                            answer='error:Num' if (check>=2**53 if result_strict else check>2**53) else encode(result)
                        counts[name]+=answer!=row['expected_bits']
                        if answer!=row['expected_bits']:misses.setdefault(name,[]).append({'id':row['id'],'args':row['args'],'expected':row['expected_bits'],'candidate':answer})
        report[fn]={'rows':len(rows),'differences':dict(counts),'counterexamples':misses}
    output=root/'bounds-models.json';output.write_text(json.dumps(report,indent=2),encoding='utf-8')
    for fn,data in report.items():print(fn,data['rows'],data['differences'])
if __name__=='__main__':main()
