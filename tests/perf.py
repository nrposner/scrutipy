import scrutipy as s
import time
t = time.time(); res = len(s.closure(3.5, 0.57, 100, 0, 7, 0.05, 0.05)); e = time.time()
print(e - t)

t = time.time(); res = len(s.closure(3.5, 1.2, 50, 0, 7, 0.05, 0.005)); e = time.time()
print(e - t)

t = time.time(); res = len(s.closure(50.9570, 28.7599, 1000, 1, 100, 0.005, 0.005)); e = time.time()
print(e - t)

