# perf Profiling Quick Reference

**Purpose**: Analyze CPU usage to find performance bottlenecks  
**Cost**: ~1-2 seconds overhead per profile  
**Output**: Text-based performance reports (no graphics needed)

---

## Three Main perf Commands

### 1. `perf stat` — Overall CPU Efficiency

**What it does**: Counts CPU cycles, cache misses, branch predictions

```bash
perf stat ./target/release/map2fig -f tests/data/cosmoglobe_clipped.fits -o /tmp/test.pdf
```

**Output shows**:
- Total CPU cycles used
- Instructions executed (and IPC = instructions per cycle)
- Cache hit/miss rates
- Branch prediction accuracy

**When to use**: Get quick overview of CPU efficiency. Lower IPC = more memory stalls. Higher cache miss % = memory bandwidth problem.

---

### 2. `perf record` — Record Full Execution

**What it does**: Records which functions use CPU time (with call stacks)

```bash
# Record with call stacks
perf record -g ./target/release/map2fig -f tests/data/cosmoglobe_clipped.fits -o /tmp/test.pdf

# View results
perf report
```

**Navigation in perf report**:
- `↓/↑`: Scroll
- `n`: Sort by samples (most common)
- `p`: Sort by percent
- `Enter`: Expand call tree
- `q`: Quit

**Output shows**:
- Percentage of time in each function
- Call stacks (who called what)

---

### 3. `perf record` with Specific Events

**What it does**: Count specific hardware events

```bash
# L1 cache misses only
perf record -e L1-dcache-load-misses ./target/release/map2fig ...

# Branch mispredictions
perf record -e branch-misses ./target/release/map2fig ...

# Memory stalls (cycles waiting for data)
perf record -e mem-stalls ./target/release/map2fig ...
```

**When to use**: Diagnose specific problems (e.g., "why are we cache misses?" or "why are so many branches mispredicted?")

---

## Common Workflow

### Quick check (30 seconds)
```bash
# 1. Overall stats
perf stat ./target/release/map2fig -f tests/data/cosmoglobe_clipped.fits -o /tmp/test.pdf 2>&1 | tail -20

# Look for:
# - IPC close to 2.0+ (good), below 1.5 (memory-bound)
# - Cache misses >30% (memory pressure)
```

### Deep dive (2-3 minutes)
```bash
# 1. Record with call stacks
perf record -q -g ./target/release/map2fig -f tests/data/cosmoglobe_clipped.fits -o /tmp/test.pdf

# 2. View top functions
perf report -n --stdio | head -50

# 3. Look for functions with high "Self" % (actual work)
# vs high "Children" % (overhead from calling others)
```

### Investigate specific function
```bash
# 1. Record
perf record -q -g -F 997 ./target/release/map2fig ...  # -F 997 = sample at 997 Hz (higher frequency)

# 2. Search for function
perf report | grep "my_function_name"

# 3. Expand its call tree (press 'Enter' on it in perf report)
```

---

## Interpreting Results

### IPC (Instructions Per Cycle)

```
3.0+ : Excellent (CPU doing lots of work per cycle)
2.0-2.5 : Good (moderate work)
<1.5 : Poor (memory-bound, CPU idle waiting for data)
```

### Cache Miss Rate

```
< 5%  : Excellent data locality
5-20% : Good cache behavior
>30%  : Significant memory pressure, might optimize
```

### Top Functions

**High "Self" % = Direct work**
- Function spends cycles doing computation
- Can optimize by making algorithm faster

**High "Children" % but low "Self" %**
- Function is overhead (setup, cleanup, calls)
- Might optimize by reducing calls or batching

---

## System Setup

### Enable perf (One-time setup)

```bash
# Current setting (restrictive = 4, permissive = 1)
cat /proc/sys/kernel/perf_event_paranoid

# Temporarily allow (until reboot)
sudo sysctl -w kernel.perf_event_paranoid=1

# Permanently allow (add to /etc/sysctl.conf)
sudo bash -c 'echo "kernel.perf_event_paranoid = 1" >> /etc/sysctl.conf'
```

### Install perf (if not present)

```bash
# Ubuntu/Debian
sudo apt install linux-tools-generic

# Fedora/RHEL
sudo dnf install perf

# Arch
sudo pacman -S perf
```

---

## Troubleshooting

### "Permission denied" / "Paranoid" error
→ Follow "Enable perf" section above, use `sudo`

### "Couldn't record kernel symbols"
→ Normal warning. You still get userspace symbols (cairo, libm, your code). Add `-k no-vmlinux` to silence.

### Symbols showing as `0x00007f1234...` (hex addresses)
→ PIE (Position Independent Executable) makes symbols hard to resolve. This is normal with modern binaries. Look for function names that ARE resolved (cairo functions, libm functions), or compile with `debuginfo=1` in Cargo.toml

### Profile taking too long
→ Use `-e cycles:P` (less frequent sampling) or reduce file size (use smaller FITS file)

---

## Example: Using This on Your FITS Files

```bash
cd /home/dwatts/projects/healpix_plotter

# Profile on a small file (fast, <1s)
perf stat ./target/release/map2fig -f tests/data/m_test.fits -o /tmp/test.pdf

# Profile on a medium file (2-3s total)
perf record -q -g ./target/release/map2fig -f tests/data/cosmoglobe_clipped.fits -o /tmp/test.pdf
perf report

# Compare two different render modes
perf record -q -g ./target/release/map2fig -f tests/data/cosmoglobe_clipped.fits -o /tmp/pdf.pdf
mv perf.data perf_pdf.data

perf record -q -g ./target/release/map2fig -f tests/data/cosmoglobe_clipped.fits -o /tmp/png.png
mv perf.data perf_png.data

# Compare
perf report --stdio < perf_pdf.data | head -30
perf report --stdio < perf_png.data | head -30
```

---

## When to Profile Again

1. **After major optimization**: Before/after comparison
2. **Performance regression**: Something got slower, perf identifies what
3. **Different data types**: FITS files with different properties (many UNSEEN pixels, different column types)
4. **New platform**: Different CPU architecture has different cache/IPC characteristics

---

## References

- `man perf` — Full documentation
- `perf list` — Available events on your system
- Linux Perf Events tutorial: https://perf.wiki.kernel.org/
