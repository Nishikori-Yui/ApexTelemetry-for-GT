import React, { useEffect, useState } from 'react'

interface Point {
    x: number
    z: number
}

interface TrackMapProps {
    currentPos?: {
        x?: number
        z?: number
        rotation?: number // Yaw in degrees
    }
    trackGeometry?: {
        view_box: string
        path_d: string
    }
    className?: string
}

export const TrackMap: React.FC<TrackMapProps> = ({ currentPos, trackGeometry, className }) => {
    const [path, setPath] = useState<Point[]>([])
    // Remove viewBox state, calculate on fly for 0-lag sync
    const boundsRef = React.useRef({ minX: Infinity, maxX: -Infinity, minZ: Infinity, maxZ: -Infinity })
    // Keep track of last valid rotation to avoid snapping to 0 when stationary
    const lastRotationRef = React.useRef(0)

    useEffect(() => {
        if (currentPos?.x !== undefined && currentPos?.z !== undefined) {
            setPath(prev => {
                const newPoint = { x: currentPos.x!, z: currentPos.z! }

                // Update bounds incrementally
                boundsRef.current.minX = Math.min(boundsRef.current.minX, newPoint.x)
                boundsRef.current.maxX = Math.max(boundsRef.current.maxX, newPoint.x)
                boundsRef.current.minZ = Math.min(boundsRef.current.minZ, newPoint.z)
                boundsRef.current.maxZ = Math.max(boundsRef.current.maxZ, newPoint.z)

                // Simple distance filter to avoid too many points
                if (prev.length > 0) {
                    const last = prev[prev.length - 1]
                    const dist = Math.sqrt(Math.pow(newPoint.x - last.x, 2) + Math.pow(newPoint.z - last.z, 2))
                    if (dist < 1.0) return prev // Skip if moved less than 1 meter
                }
                return [...prev, newPoint]
            })
        }
    }, [currentPos?.x, currentPos?.z])

    // Memoize track geometry bounds parsing
    const trackGeoBounds = React.useMemo(() => {
        if (trackGeometry?.view_box) {
            const parts = trackGeometry.view_box.split(' ').map(parseFloat)
            if (parts.length === 4 && !parts.some(isNaN)) {
                return { x: parts[0], y: parts[1], w: parts[2], h: parts[3] }
            }
        }
        return null
    }, [trackGeometry?.view_box])

    // Calculate viewBox synchronously
    let { minX, maxX, minZ, maxZ } = boundsRef.current

    if (trackGeoBounds) {
        minX = Math.min(minX, trackGeoBounds.x)
        maxX = Math.max(maxX, trackGeoBounds.x + trackGeoBounds.w)
        minZ = Math.min(minZ, trackGeoBounds.y)
        maxZ = Math.max(maxZ, trackGeoBounds.y + trackGeoBounds.h)
    }

    let finalViewBox = '0 0 1000 1000'
    let currentWidth = 1000 // Fallback width for arrow calculation

    // Only calculate if we have valid bounds
    if (minX !== Infinity) {
        let width = maxX - minX
        let height = maxZ - minZ

        if (width < 10) {
            const cx = (minX + maxX) / 2
            width = 100
            minX = cx - 50
        }
        if (height < 10) {
            const cz = (minZ + maxZ) / 2
            height = 100
            minZ = cz - 50
        }

        currentWidth = width

        // Add 25% padding
        const paddingX = width * 0.25
        const paddingZ = height * 0.25
        finalViewBox = `${minX - paddingX} ${minZ - paddingZ} ${width + paddingX * 2} ${height + paddingZ * 2}`
    } else if (trackGeoBounds) {
        // Fallback to purely track geometry if no path points yet
        finalViewBox = trackGeometry!.view_box
        if (trackGeoBounds) currentWidth = trackGeoBounds.w
    }


    const pathD = path.length > 0
        ? `M ${path.map(p => `${p.x},${p.z}`).join(' L ')}`
        : ''

    let arrowPath = ''
    if (currentPos?.x !== undefined && currentPos?.z !== undefined) {
        // Arrow size: 6% of effective width.
        // User asked for constant ratio.
        const size = currentWidth * 0.06

        // Heading - use stored rotation or fallback to last known value
        let yaw = currentPos.rotation

        if (yaw !== undefined) {
            lastRotationRef.current = yaw
        } else {
            yaw = lastRotationRef.current
        }

        // Define Arrow Points (Relative to 0,0) - Pointing UP (Negative Z)
        const tip = { x: 0, z: -size }
        const left = { x: -size * 0.7, z: size }
        const notch = { x: 0, z: size * 0.6 }
        const right = { x: size * 0.7, z: size }

        // Rotate points around (0,0)
        const rad = (yaw * Math.PI) / 180
        // Rotation matrix: x' = x cos - z sin, z' = x sin + z cos
        // Wait, positive rotation (clockwise)
        const rotate = (p: Point) => ({
            x: p.x * Math.cos(rad) - p.z * Math.sin(rad),
            z: p.x * Math.sin(rad) + p.z * Math.cos(rad)
        })

        const rt = rotate(tip)
        const rl = rotate(left)
        const rn = rotate(notch)
        const rr = rotate(right)

        const cx = currentPos.x
        const cz = currentPos.z

        arrowPath = `M ${cx + rt.x},${cz + rt.z} L ${cx + rl.x},${cz + rl.z} L ${cx + rn.x},${cz + rn.z} L ${cx + rr.x},${cz + rr.z} Z`
    }

    return (
        <svg className={className} viewBox={finalViewBox} preserveAspectRatio="xMidYMid meet" style={{ overflow: 'hidden' }}>
            {trackGeometry?.path_d && (
                <path d={trackGeometry.path_d} fill="none" stroke="#444" strokeWidth="2" vectorEffect="non-scaling-stroke" />
            )}
            <path d={pathD} fill="none" stroke="cyan" strokeWidth="4" vectorEffect="non-scaling-stroke" />
            {arrowPath && (
                <path d={arrowPath} fill="red" vectorEffect="non-scaling-stroke" />
            )}
        </svg>
    )
}
