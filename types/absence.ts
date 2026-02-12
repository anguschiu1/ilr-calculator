export interface AbsencePeriod {
  start_date: string
  end_date: string
}

export interface CalculationResult {
  absence_start: string
  absence_end: string
  window_start: string
  window_end: string
  total_days_in_window: number
}
