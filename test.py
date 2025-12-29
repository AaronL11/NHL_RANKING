import matplotlib.pyplot as plt
from matplotlib.offsetbox import AnchoredText
import numpy as np
import math

rng = np.random.default_rng()

GAMES = 1
PLAYS = 56
# STOP_WIN = 
# STOP_LOSS = 0
K = PLAYS*GAMES
N = 10000
intss = [rng.integers(0,37,K) for _ in range(N)]
# intss = [[1]*idx+[0]+[1]*(56-idx-1)]
# print(len(intss))
# assert len(intss[0]) == 56
# print(intss[:3])

p=1/37
geo = np.random.geometric(p=p,size=N+1)
geo[0] = 0 


a = lambda n: max(1,math.floor((math.sqrt(1+8*n)-1)/2))
# a = lambda n: math.floor((math.sqrt(1+8*n)-1)/2) if n < 56 else 10+(n-56)
s = lambda n: sum(a(k) for k in range(1,n+1))
sk = lambda n,k: k*s(n)
S = s(PLAYS)
# S = s(PLAYS*GAMES)

DEF_BAL = GAMES*S

balances = [[DEF_BAL] for _ in range(N)]

Xs = list(range(K+1))

for balance,ints in zip(balances,intss):
    b=1
    A=balance[-1]
    # k=1
    k=A/DEF_BAL
    played = 0
    c=(A-GAMES*S)/PLAYS
    # c=0
    won = 0
    lost = 0
    spin = 0
    for n in ints:
        spin += 1
        if balance[-1] == 0:
            balance.append(0)
            continue
        elif won == GAMES:# or lost == GAMES//3-1:
            balance.append(balance[-1])
            continue
        b = 1*a(spin)
        # b = 1*a(spin)
        # n_bal = balance[-1]
        n_bal = balance[-1]
        n_bal -= b
        # print(f'{spin=}\n{n_bal=}\n{a(spin),b=}\n{DEF_BAL-s(spin)=}')
        if n == 0:
            won += 1
            played += 1
            lost = 0
            # n_bal = 36*a(spin) - s(spin)
            # n_bal = DEF_BAL + 36*a(spin) - s(spin)
            n_bal += 36*b
            # A=balance[-1]
            # k=A/DEF_BAL
            # c=(A-GAMES*S)/PLAYS
            # print(f'\n{spin=}\n{a(spin),s(spin)=}\n{n_bal=}\n{b=}\n{DEF_BAL+36*b-s(spin)=}')
            spin = 1
        else:
            # n_bal = DEF_BAL - s(spin)
            lost += 1
            if spin%60 == 0:
                played += 1
                spin = 1
            # print(f'\n{spin=}\n{a(spin),s(spin)=}\n{n_bal=}\n{b=}\n{DEF_BAL+36*b-s(spin)=}')
            #     spin = 1
        balance.append(n_bal)
    # for n in geo:
    #     balance.append(balance[-1]+36*a(n)-s(n))
        # if n==1:
        #     balance[-1] += 36*b
        #     b=1;i=1;k=1
        # if i==b+1:
        #     b+=1
        #     i=0
        # i+=1
        # k+=1
        # if k==57:
        #     b=1;i=1;k=1

avg_balance = [np.mean([balance[i] for balance in balances]) for i in range(K+1)]

count_top,count_bot =0,0
for i in range(N):
    if balances[i][-1] > DEF_BAL:
        count_top += 1
    else:
        count_bot += 1

for balance in balances:
# for balance in balances:
#     plt.plot(geo,balance)
# plt.plot(geo,avg_balance,linewidth=3.0)
# plt.plot(geo,[2000]*(K+1),linewidth=3.0,color=(0,0,0))
    plt.plot(Xs,balance,linewidth=1)
plt.plot(Xs,avg_balance,linewidth=2.0,color=(0,0,0),label=f'Average final balance = {avg_balance[-1]:.2f}')
plt.plot(Xs,[DEF_BAL]*(K+1),linewidth=2.0,color=(0,0,0))

# plt.legend()

text = AnchoredText(
    f'Average final balance = {avg_balance[-1]:.2f}\n{count_top} gamblers made money, {count_bot} lost money, (win rate {count_top/N*100:.2f}%)',
    frameon=True,
    loc=4,
    pad=0.5
)

plt.setp(text.patch, facecolor='white', alpha=0.5)
plt.gca().add_artist(text)

plt.ylabel('Account Balance ($)')
plt.xlabel('number of spins')
plt.title(f"Simulated returns for {GAMES} game{'' if GAMES==1 else 's'}\n{N=}\nStarting balance ${DEF_BAL}")

plt.show()

outcomes = []
freq = {}
for balance in balances:
    outcome = balance[-1] - avg_balance[-1]
    if outcome in freq:
        freq[outcome] += 1
    else:
        freq[outcome] = 1
        outcomes.append(outcome)
outcomes.sort()
counts = [freq[outcome] for outcome in outcomes]

mu,med,sigma = np.mean(outcomes),np.median(outcomes),np.std(outcomes)
plt.title(
    f'Distribution of earnings\nmean: {"+" if np.sign(mu)==1 else "-"}${mu:.2f}, median {"+" if np.sign(mu)==1 else "-"}${med:.2f},variance: ±${sigma:.2f}'
    )
plt.ylabel('Frequency')
plt.xlabel('Earnings ($)')
n_bins = N//100
plt.gca().hist(outcomes,bins=n_bins)
plt.axvline(x=mu,linewidth=3.0,color='black',label='mean')
plt.axvline(x=med,linewidth=3.0,color='red',label='median')
plt.legend()
plt.show()