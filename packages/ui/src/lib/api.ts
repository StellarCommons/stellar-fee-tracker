import type {
  CurrentFeeResponse,
  FeeHistoryResponse,
  FeeTrendResponse,
  InsightsResponse,
  HealthResponse,
} from './types'

const BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'

async function get<T>(path: string): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    next: { revalidate: 0 }, // always fresh
  })
  if (!res.ok) {
    throw new Error(`API error ${res.status} on ${path}`)
  }
  return res.json() as Promise<T>
}

export const api = {
  currentFees:  ()               => get<CurrentFeeResponse>('/fees/current'),
  feeHistory:   (window = '1h')  => get<FeeHistoryResponse>(`/fees/history?window=${window}`),
  feeTrend:     ()               => get<FeeTrendResponse>('/fees/trend'),
  insights:     ()               => get<InsightsResponse>('/insights'),
  health:       ()               => get<HealthResponse>('/health'),
}