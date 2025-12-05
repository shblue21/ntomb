# Design Document: Graveyard Adaptive Layout

## Overview

This design document describes the implementation of adaptive endpoint node layout for ntomb's Graveyard widget. The feature dynamically adjusts ring radii based on available canvas dimensions, allowing endpoint nodes to spread out and utilize screen space more effectively on larger terminals while maintaining readability on smaller screens.

## Architecture

The adaptive layout system integrates into the existing Graveyard rendering pipeline:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Graveyard Render Pipeline                     │
├─────────────────────────────────────────────────────────────────┤
│  1. Canvas Area Calculation (from Rect)                         │
│           ↓                                                      │
│  2. Layout Parameters Calculation (NEW)                         │
│     - Determine if adaptive mode applies                        │
│     - Calculate scaled ring radii                               │
│           ↓                                                      │
│  3. Endpoint Position Calculation                               │
│     - Use adaptive radii instead of fixed constants             │
│           ↓                                                      │
│  4. Canvas Rendering                                            │
│     - Draw rings, edges, nodes at calculated positions          │
└─────────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### 1. LayoutConfig Struct

A new struct to hold calculated layout parameters for a given canvas size:

```rust
/// Layout configuration calculated from canvas dimensions
/// 
/// Contains the ring radii and other layout parameters that adapt
/// to the available canvas space.
pub struct LayoutConfig {
    /// Radius for Low latency ring (innermost)
    pub ring_low: f64,
    /// Radius for Medium latency ring (middle)
    pub ring_medium: f64,
    /// Radius for High latency ring (outermost)
    pub ring_high: f64,
    /// Minimum padding from canvas edges
    pub edge_padding: f64,
    /// Whether adaptive mode is active (vs fixed fallback)
    pub is_adaptive: bool,
}
```

### 2. calculate_layout_config Function

```rust
/// Calculate layout configuration based on canvas dimensions
/// 
/// # Arguments
/// * `canvas_width` - Width of the canvas in canvas units (typically 100.0)
/// * `canvas_height` - Height of the canvas in canvas units (scaled from terminal rows)
/// 
/// # Returns
/// LayoutConfig with appropriate ring radii for the given dimensions
pub fn calculate_layout_config(canvas_width: f64, canvas_height: f64) -> LayoutConfig
```

### 3. Updated calculate_endpoint_position Function

The existing function will be modified to accept `LayoutConfig` instead of using hardcoded `RING_RADII`:

```rust
/// Calculate endpoint position on the canvas based on latency bucket
/// 
/// # Arguments
/// * `endpoint_idx` - Index of this endpoint within its latency bucket
/// * `total_in_bucket` - Total number of endpoints in the same bucket
/// * `latency_bucket` - The latency classification for ring selection
/// * `layout` - Layout configuration with calculated ring radii
/// 
/// # Returns
/// (x, y) coordinates in canvas space
pub fn calculate_endpoint_position(
    endpoint_idx: usize,
    total_in_bucket: usize,
    latency_bucket: LatencyBucket,
    layout: &LayoutConfig,
) -> (f64, f64)
```

## Data Models

### Constants

```rust
/// Minimum canvas dimension (in canvas units) to enable adaptive layout
/// Below this threshold, fixed radii are used for readability
const ADAPTIVE_THRESHOLD: f64 = 60.0;

/// Default fixed ring radii for small screens (current behavior)
const DEFAULT_RING_RADII: [f64; 3] = [15.0, 25.0, 35.0];

/// Ring ratio multipliers (Low:Medium:High approximately 3:5:7)
/// These ratios are preserved when scaling adaptively
const RING_RATIO_LOW: f64 = 0.30;
const RING_RATIO_MEDIUM: f64 = 0.50;
const RING_RATIO_HIGH: f64 = 0.70;

/// Edge padding as percentage of available radius
const EDGE_PADDING_PERCENT: f64 = 0.10;

/// Minimum edge padding in canvas units
const MIN_EDGE_PADDING: f64 = 5.0;
```

### Layout Calculation Logic

```
Available Radius = min(canvas_width, canvas_height) / 2 - edge_padding

If Available Radius < ADAPTIVE_THRESHOLD / 2:
    Use DEFAULT_RING_RADII (fixed mode)
Else:
    ring_low = Available Radius * RING_RATIO_LOW
    ring_medium = Available Radius * RING_RATIO_MEDIUM  
    ring_high = Available Radius * RING_RATIO_HIGH
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

Based on the prework analysis, the following correctness properties have been identified:

### Property 1: Adaptive Scaling

*For any* canvas dimensions where the smaller dimension exceeds the adaptive threshold, the calculated ring radii SHALL scale proportionally to the smaller dimension, and different canvas sizes SHALL produce different ring radii.

**Validates: Requirements 1.1, 1.2, 3.1**

### Property 2: Fixed Fallback

*For any* canvas dimensions where the smaller dimension is below the adaptive threshold, the calculated ring radii SHALL equal the default fixed values (15.0, 25.0, 35.0).

**Validates: Requirements 1.3**

### Property 3: Bounds Invariant

*For any* canvas dimensions and any calculated endpoint position, the coordinates SHALL remain within the canvas bounds with appropriate padding (x and y in range [padding, 100-padding]).

**Validates: Requirements 1.4, 3.2**

### Property 4: Ring Ratio Preservation

*For any* canvas dimensions in adaptive mode, the ratios between ring radii SHALL remain constant (ring_low < ring_medium < ring_high, with consistent proportional spacing).

**Validates: Requirements 2.1, 2.3**

### Property 5: Angular Separation

*For any* number of endpoints in a latency bucket, the angular separation between adjacent nodes SHALL be at least (2π / max_nodes_per_ring) radians.

**Validates: Requirements 2.2**

## Error Handling

- **Invalid canvas dimensions**: If canvas width or height is zero or negative, fall back to default fixed radii
- **Empty endpoint list**: No special handling needed; existing code handles this case
- **Extreme aspect ratios**: The smaller dimension is always used, preventing overflow in either direction

## Testing Strategy

### Property-Based Testing

The implementation will use the `proptest` crate for property-based testing, as established in the existing codebase.

Each correctness property will be implemented as a property-based test:

1. **Adaptive Scaling Test**: Generate random canvas dimensions above threshold, verify radii scale with min dimension
2. **Fixed Fallback Test**: Generate random canvas dimensions below threshold, verify fixed radii are returned
3. **Bounds Invariant Test**: Generate random canvas dimensions and endpoint configurations, verify all positions are within bounds
4. **Ring Ratio Test**: Generate random canvas dimensions, verify ratio between rings is preserved
5. **Angular Separation Test**: Generate random endpoint counts, verify minimum angular separation

### Unit Tests

Unit tests will cover:
- Edge cases: zero dimensions, very small canvases, very large canvases
- Specific known inputs with expected outputs for regression testing
- Integration with existing `calculate_endpoint_position` function

### Test Configuration

- Property tests will run a minimum of 100 iterations each
- Each property test will be tagged with the format: `**Feature: graveyard-adaptive-layout, Property {number}: {property_text}**`

