#!/usr/bin/env node
/**
 * Node script to test the WASM build.
 * Run after: wasm-pack build --target nodejs --out-dir pkg
 *
 * Usage: node scripts/wasm-test.mjs
 *    or: pnpm run wasm:test
 */

import { fileURLToPath, pathToFileURL } from 'url'
import { dirname, join } from 'path'
import { existsSync } from 'fs'

const __dirname = dirname(fileURLToPath(import.meta.url))
const pkgPath = join(__dirname, '..', 'pkg', 'ilr_calculator.js')

if (!existsSync(pkgPath)) {
  console.error('WASM pkg not found. Build it first:')
  console.error('  pnpm run wasm:build:node')
  console.error('  or: wasm-pack build --target nodejs --out-dir pkg')
  process.exit(1)
}

async function run() {
  let passed = 0
  let failed = 0

  const pkgUrl = pathToFileURL(pkgPath).href
  const pkg = await import(pkgUrl)
  const { calculate_rolling_absences, validate_absence_periods } = pkg.default || pkg

  // Test 1: calculate_rolling_absences with valid input
  try {
    const input = JSON.stringify([
      { start_date: '2023-01-01', end_date: '2023-01-10' }
    ])
    const result = calculate_rolling_absences(input)
    const results = JSON.parse(result)
    if (results.length !== 1 || results[0].total_days_in_window !== 10) {
      throw new Error(`Expected 1 result with 10 days, got ${JSON.stringify(results)}`)
    }
    console.log('✓ calculate_rolling_absences (valid single period)')
    passed++
  } catch (e) {
    console.error('✗ calculate_rolling_absences (valid):', e.message)
    failed++
  }

  // Test 2: calculate_rolling_absences with invalid JSON (should throw)
  try {
    calculate_rolling_absences('not json')
    console.error('✗ calculate_rolling_absences (invalid JSON): expected throw')
    failed++
  } catch (e) {
    console.log('✓ calculate_rolling_absences (invalid JSON throws)')
    passed++
  }

  // Test 3: calculate_rolling_absences with two periods
  try {
    const input = JSON.stringify([
      { start_date: '2023-01-01', end_date: '2023-01-10' },
      { start_date: '2023-08-01', end_date: '2023-08-20' }
    ])
    const result = calculate_rolling_absences(input)
    const results = JSON.parse(result)
    if (results.length !== 2) throw new Error(`Expected 2 results, got ${results.length}`)
    if (results[0].total_days_in_window !== 10) throw new Error(`First period expected 10 days, got ${results[0].total_days_in_window}`)
    if (results[1].total_days_in_window !== 30) throw new Error(`Second period expected 30 days, got ${results[1].total_days_in_window}`)
    console.log('✓ calculate_rolling_absences (two periods)')
    passed++
  } catch (e) {
    console.error('✗ calculate_rolling_absences (two periods):', e.message)
    failed++
  }

  // Test 4: validate_absence_periods valid
  try {
    const input = JSON.stringify([
      { start_date: '2023-01-01', end_date: '2023-01-10' }
    ])
    const result = validate_absence_periods(input)
    const parsed = JSON.parse(result)
    if (!Array.isArray(parsed) || parsed.length !== 1) throw new Error('Expected 1 period')
    console.log('✓ validate_absence_periods (valid)')
    passed++
  } catch (e) {
    console.error('✗ validate_absence_periods (valid):', e.message)
    failed++
  }

  // Test 5: validate_absence_periods invalid (end before start) - should throw
  try {
    validate_absence_periods(
      JSON.stringify([{ start_date: '2023-06-01', end_date: '2023-05-20' }])
    )
    console.error('✗ validate_absence_periods (invalid): expected throw')
    failed++
  } catch (e) {
    console.log('✓ validate_absence_periods (invalid period throws)')
    passed++
  }

  console.log('')
  console.log(`WASM tests: ${passed} passed, ${failed} failed`)
  process.exit(failed > 0 ? 1 : 0)
}

run().catch((e) => {
  console.error(e)
  process.exit(1)
})
