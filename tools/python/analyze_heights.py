#!/usr/bin/env python3
"""Analyze colorbar height to verify the height % 3 constraint hypothesis."""

def analyze_heights():
    """Check various image widths and their resulting colorbar heights."""
    
    print("=" * 70)
    print("COLORBAR HEIGHT ANALYSIS")
    print("=" * 70)
    print()
    
    # Test various image widths
    widths = [800, 900, 1000, 1200, 1440, 1600, 1920]
    
    for width in widths:
        outer_pad = 24
        map_w = width - 2 * outer_pad
        map_h = map_w / 2.0
        cbar_h_float = map_h / 20.0
        cbar_h_rounded = round(cbar_h_float)
        cbar_h_int = int(cbar_h_float)
        
        mod3_rounded = cbar_h_rounded % 3
        mod3_int = cbar_h_int % 3
        
        status_rounded = "✓ DIVISIBLE BY 3" if mod3_rounded == 0 else f"✗ PROBLEMATIC (mod 3 = {mod3_rounded})"
        status_int = "✓ DIVISIBLE BY 3" if mod3_int == 0 else f"✗ PROBLEMATIC (mod 3 = {mod3_int})"
        
        print(f"Image width: {width}px")
        print(f"  map_w = {width} - 48 = {map_w}")
        print(f"  map_h = {map_w} / 2 = {map_h}")
        print(f"  cbar_h (float) = {map_h} / 20 = {cbar_h_float}")
        print(f"  cbar_h (rounded) = {cbar_h_rounded}  {status_rounded}")
        print(f"  cbar_h (truncated) = {cbar_h_int}  {status_int}")
        print()
    
    print("=" * 70)
    print("RECOMMENDED FIXES")
    print("=" * 70)
    print()
    print("Option 1: Adjust divisor to ensure height % 3 == 0")
    print("  Instead of cbar_h = map_h / 20.0")
    print("  Use cbar_h that ensures multiple of 3")
    print()
    
    print("Option 2: Round to nearest multiple of 3")
    print("  cbar_h = (map_h / 20.0).round()")
    print("  cbar_h = ((cbar_h + 1.5).floor() / 3.0 * 3.0).round()")
    print()
    
    print("Option 3: Use specific heights that work well")
    print("  Try: 24, 27, 30, 33, 36, 39, 42")
    print()

if __name__ == "__main__":
    analyze_heights()
