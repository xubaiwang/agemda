

# Syntax

Each todo has 4 properties:

1. `summary`
2. `start`: start date
3. `due`: expected complete date
4. `completed`: actual complete date

## Complete Form

```md
- [ ] Todo 1 summary <agmd:start=2025-03-09;due=2025-03-12>
- [x] Todo 2 summary <agmd:start=2025-03-09;due=2025-03-12;completed=2025-03-10>

- [x] completed snaps to due || start <agmd:start=2025-03-09;due=2025-03-12>
- [ ] start and due is optional (associated with each day) <agmd:>
- [x] done without completed is ignored <agmd:>
```

## Base for defaults

```md
- [ ] the full year <agmd:2025>
- [ ] the full month <agmd:2025-03>
- [ ] the day <agmd:2025-03-03>
- [ ] override due day <agmd:2025-03;due=10>
- [ ] override due year <agmd:2025-03-20;due=2026>
```

## Duration

Currently not implemented.

```md
~- [ ] the month <agmd:2025-03;duration=P1M>~
```
