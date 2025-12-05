# Requirements Document

## Introduction

This document defines the requirements for adaptive endpoint node layout in ntomb's Graveyard (network topology map) widget. Currently, nodes are positioned on fixed-radius rings (15, 25, 35 in canvas units), causing them to cluster near the center regardless of screen size. This feature enables nodes to spread out proportionally when the canvas area is sufficiently large, making better use of available screen real estate.

## Glossary

- **Graveyard**: The main network topology visualization canvas in ntomb
- **Endpoint Node**: A visual node representing a remote IP address (displayed with icons like 🎃, 🪦, ⚰️)
- **HOST Center**: The central node representing the local host, positioned at canvas center (coffin shape ⚰️)
- **Latency Ring**: Concentric circles around HOST where nodes are positioned based on latency bucket (Low/Medium/High)
- **Canvas Space**: Virtual coordinate space ranging from 0-100 in both X and Y dimensions
- **Ring Radius**: Distance from HOST center to each latency ring
- **Adaptive Layout**: Dynamic adjustment of layout parameters based on available canvas dimensions
- **Aspect Ratio**: The ratio of canvas width to height

## Requirements

### Requirement 1

**User Story:** As a user with a large terminal, I want endpoint nodes to spread out across the available canvas space, so that I can see the network topology more clearly without nodes clustering in the center.

#### Acceptance Criteria

1. WHEN the canvas area exceeds a minimum size threshold THEN the Graveyard SHALL calculate ring radii proportionally to the smaller of canvas width or height
2. WHEN the canvas dimensions change THEN the Graveyard SHALL recalculate endpoint positions to utilize the new available space
3. WHILE the canvas is smaller than the minimum threshold THEN the Graveyard SHALL use the default fixed ring radii (15, 25, 35) to maintain readability on small screens
4. WHEN calculating adaptive ring radii THEN the Graveyard SHALL maintain a minimum padding from canvas edges to prevent label clipping

### Requirement 2

**User Story:** As a user, I want the relative spacing between latency rings to be preserved when the layout adapts, so that I can still distinguish between Low, Medium, and High latency endpoints.

#### Acceptance Criteria

1. WHEN adaptive layout is applied THEN the Graveyard SHALL maintain proportional spacing ratios between the three latency rings
2. WHEN endpoints are positioned on rings THEN the Graveyard SHALL ensure adequate angular separation between adjacent nodes to minimize overlap
3. WHEN the canvas is resized THEN the Graveyard SHALL preserve the visual hierarchy where Low latency nodes appear closest to HOST and High latency nodes appear furthest

### Requirement 3

**User Story:** As a user, I want the layout to handle various terminal aspect ratios gracefully, so that the network map remains usable in both wide and tall terminal configurations.

#### Acceptance Criteria

1. WHEN the canvas has a non-square aspect ratio THEN the Graveyard SHALL use the smaller dimension to determine maximum ring radius to prevent nodes from being clipped
2. WHEN nodes are positioned THEN the Graveyard SHALL clamp coordinates to stay within canvas bounds with appropriate padding

