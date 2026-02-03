export const SPEED_ARC_START = -120
export const SPEED_ARC_SPAN = 240
export const SPEED_ARC_PATH = describeArc(
  110,
  110,
  90,
  SPEED_ARC_START,
  SPEED_ARC_START + SPEED_ARC_SPAN,
)

export const FUEL_GAUGE_START = 270
export const FUEL_GAUGE_SPAN = 180
export const FUEL_GAUGE_CX = 130
export const FUEL_GAUGE_CY = 130
export const FUEL_GAUGE_R = 90
export const FUEL_GAUGE_PATH = describeArcWithSweep(
  FUEL_GAUGE_CX,
  FUEL_GAUGE_CY,
  FUEL_GAUGE_R,
  FUEL_GAUGE_START,
  FUEL_GAUGE_START - FUEL_GAUGE_SPAN,
  1,
)

export const TACH_MAX_RPM = 12000
export const TACH_RED_START = 9000
export const TACH_ARC_START = -120
export const TACH_ARC_SPAN = 240
export const TACH_ARC_PATH = describeArc(
  110,
  110,
  90,
  TACH_ARC_START,
  TACH_ARC_START + TACH_ARC_SPAN,
)

export const TURBO_START = 180
export const TURBO_SWEEP = 270
export const TURBO_NEG_SWEEP = 90
export const TURBO_POS_SWEEP = 180
export const TURBO_CX = 130
export const TURBO_CY = 100
export const TURBO_CY_ACTUAL = 100
export const TURBO_R = 90
export const TURBO_BASE_PATH = describeArc(
  TURBO_CX,
  TURBO_CY,
  TURBO_R,
  TURBO_START,
  TURBO_START + TURBO_SWEEP,
)

export function buildSpeedTicks(
  maxValue: number,
  majorStep: number,
  minorStep: number,
) {
  const ticks: Array<{
    value: number
    kind: 'major' | 'minor'
    x1: number
    y1: number
    x2: number
    y2: number
  }> = []
  const labels: Array<{ value: number; x: number; y: number }> = []
  for (let value = 0; value <= maxValue + 0.1; value += minorStep) {
    const isMajor = value % majorStep === 0
    const kind = isMajor ? 'major' : 'minor'
    const angle = SPEED_ARC_START + (value / maxValue) * SPEED_ARC_SPAN
    const outer = polarToCartesian(110, 110, 104, angle)
    const inner = polarToCartesian(110, 110, isMajor ? 84 : 92, angle)
    ticks.push({
      value,
      kind,
      x1: inner.x,
      y1: inner.y,
      x2: outer.x,
      y2: outer.y,
    })
    if (isMajor) {
      const labelPos = polarToCartesian(110, 110, 70, angle)
      labels.push({ value, x: labelPos.x, y: labelPos.y })
    }
  }
  return { ticks, labels }
}

export function buildTachTicks() {
  const majorStep = 1000
  const minorStep = 500
  const labelStep = 3000
  const ticks: Array<{
    kind: 'major' | 'minor'
    x1: number
    y1: number
    x2: number
    y2: number
  }> = []
  const labels: Array<{ value: number; x: number; y: number }> = []
  for (let value = 0; value <= TACH_MAX_RPM + 0.1; value += minorStep) {
    const isMajor = value % majorStep === 0
    const kind = isMajor ? 'major' : 'minor'
    const angle = TACH_ARC_START + (value / TACH_MAX_RPM) * TACH_ARC_SPAN
    const outer = polarToCartesian(110, 110, 104, angle)
    const inner = polarToCartesian(110, 110, isMajor ? 84 : 92, angle)
    ticks.push({
      kind,
      x1: inner.x,
      y1: inner.y,
      x2: outer.x,
      y2: outer.y,
    })
    if (isMajor && value % labelStep === 0) {
      const labelPos = polarToCartesian(110, 110, 70, angle)
      labels.push({ value: value / 1000, x: labelPos.x, y: labelPos.y })
    }
  }
  return { ticks, labels }
}

export function turboAngleForNorm(valueNorm: number) {
  return 270 + valueNorm * 90
}

export function buildTurboTicks() {
  const marks = [-1, 0, 1, 2]
  return marks.map((mark) => {
    const angle = turboAngleForNorm(mark)
    const outer = polarToCartesian(TURBO_CX, TURBO_CY, TURBO_R + 12, angle)
    const inner = polarToCartesian(TURBO_CX, TURBO_CY, TURBO_R - 8, angle)
    const labelPos = polarToCartesian(TURBO_CX, TURBO_CY, TURBO_R - 24, angle)
    return {
      label: mark.toString(),
      x1: inner.x,
      y1: inner.y,
      x2: outer.x,
      y2: outer.y,
      tx: labelPos.x,
      ty: labelPos.y,
      major: true,
    }
  })
}

export function buildTurboMinorTicks() {
  const minorMarks = [-0.75, -0.5, -0.25, 0.5, 1.5]
  return minorMarks.map((mark) => {
    const angle = turboAngleForNorm(mark)
    const outer = polarToCartesian(TURBO_CX, TURBO_CY, TURBO_R + 8, angle)
    const inner = polarToCartesian(TURBO_CX, TURBO_CY, TURBO_R - 4, angle)
    return { x1: inner.x, y1: inner.y, x2: outer.x, y2: outer.y }
  })
}

export function buildFuelTicks() {
  const ticks: { x1: number; y1: number; x2: number; y2: number; major: boolean }[] = []
  for (let i = 0; i <= 10; i++) {
    const ratio = i / 10
    const angle = FUEL_GAUGE_START + ratio * FUEL_GAUGE_SPAN
    const isMajor = i === 0 || i === 5 || i === 10
    const outerR = FUEL_GAUGE_R + 6
    const innerR = isMajor ? FUEL_GAUGE_R - 18 : FUEL_GAUGE_R - 10
    const outer = polarToCartesian(FUEL_GAUGE_CX, FUEL_GAUGE_CY, outerR, angle)
    const inner = polarToCartesian(FUEL_GAUGE_CX, FUEL_GAUGE_CY, innerR, angle)
    ticks.push({
      x1: outer.x,
      y1: outer.y,
      x2: inner.x,
      y2: inner.y,
      major: isMajor,
    })
  }
  return ticks
}

function polarToCartesian(
  cx: number,
  cy: number,
  radius: number,
  angleInDegrees: number,
) {
  const angleInRadians = ((angleInDegrees - 90) * Math.PI) / 180.0
  return {
    x: cx + radius * Math.cos(angleInRadians),
    y: cy + radius * Math.sin(angleInRadians),
  }
}

export function describeArc(
  cx: number,
  cy: number,
  radius: number,
  startAngle: number,
  endAngle: number,
) {
  const start = polarToCartesian(cx, cy, radius, endAngle)
  const end = polarToCartesian(cx, cy, radius, startAngle)
  const sweep = endAngle - startAngle
  const largeArcFlag = sweep <= 180 ? '0' : '1'
  return `M ${start.x} ${start.y} A ${radius} ${radius} 0 ${largeArcFlag} 0 ${end.x} ${end.y}`
}

export function describeArcWithSweep(
  cx: number,
  cy: number,
  radius: number,
  startAngle: number,
  endAngle: number,
  sweepFlag: 0 | 1,
) {
  const start = polarToCartesian(cx, cy, radius, startAngle)
  const end = polarToCartesian(cx, cy, radius, endAngle)
  const sweep = endAngle - startAngle
  const largeArcFlag = Math.abs(sweep) <= 180 ? '0' : '1'
  return `M ${start.x} ${start.y} A ${radius} ${radius} 0 ${largeArcFlag} ${sweepFlag} ${end.x} ${end.y}`
}
