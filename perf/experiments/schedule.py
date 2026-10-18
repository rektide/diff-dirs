"""Seeded balanced blocks: each permutation followed immediately by its reverse."""
import random


def schedule(names, blocks, seed):
    if len(names) < 2 or len(set(names)) != len(names):
        raise ValueError("at least two distinct variants required")
    if blocks < 2 or blocks % 2:
        raise ValueError("blocks must be positive and even: complete permutation/reverse pairs")
    rng = random.Random(seed)
    result = []
    for _ in range(blocks // 2):
        order = list(names)
        rng.shuffle(order)
        result.extend([order, list(reversed(order))])
    return result
