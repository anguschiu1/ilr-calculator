<template>
  <div class="min-h-screen bg-background dark">
    <div class="container mx-auto px-4 py-6 sm:px-6 md:px-8 lg:px-12">
      <!-- Header Section -->
      <div class="mb-8 space-y-2 text-center">
        <h1 class="text-3xl font-bold tracking-tight sm:text-4xl md:text-5xl">
          ILR Calculator
        </h1>
        <p class="text-muted-foreground text-lg">
          Calculate rolling absence periods using a 365-day window
        </p>
      </div>

      <!-- Main Card -->
      <Card class="mx-auto max-w-2xl">
        <CardHeader>
          <CardTitle class="text-2xl">Absence Calculator</CardTitle>
          <CardDescription>
            Enter absence periods to calculate total days within rolling 365-day windows
          </CardDescription>
        </CardHeader>

        <CardContent class="space-y-6">
          <!-- Absence Periods Input -->
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <Label class="text-base font-semibold">Absence Periods</Label>
              <Button
                variant="outline"
                size="sm"
                @click="addPeriod"
                class="min-h-[44px]"
              >
                + Add Period
              </Button>
            </div>

            <div v-if="periods.length === 0" class="rounded-lg border border-dashed p-8 text-center">
              <p class="text-muted-foreground mb-4">No absence periods added</p>
              <p class="text-muted-foreground text-sm">
                Click "Add Period" to get started
              </p>
            </div>

            <div v-else class="space-y-4">
              <div
                v-for="(period, index) in periods"
                :key="index"
                class="grid grid-cols-1 gap-4 rounded-lg border p-4 sm:grid-cols-2"
              >
                <div class="space-y-2">
                  <Label :for="`start-${index}`">Start Date</Label>
                  <Input
                    :id="`start-${index}`"
                    v-model="period.start_date"
                    type="date"
                    class="w-full"
                    @input="validatePeriod(index)"
                  />
                </div>
                <div class="space-y-2">
                  <Label :for="`end-${index}`">End Date</Label>
                  <div class="flex gap-2">
                    <Input
                      :id="`end-${index}`"
                      v-model="period.end_date"
                      type="date"
                      class="flex-1"
                      @input="validatePeriod(index)"
                    />
                    <Button
                      variant="ghost"
                      size="icon"
                      @click="removePeriod(index)"
                      class="min-h-[44px] min-w-[44px]"
                    >
                      ×
                    </Button>
                  </div>
                </div>
                <div v-if="errors[index]" class="col-span-2 text-sm text-destructive">
                  {{ errors[index] }}
                </div>
              </div>
            </div>
          </div>

          <!-- Options Section -->
          <div class="space-y-4 rounded-lg border p-4">
            <h3 class="text-lg font-semibold">Options</h3>
            <div class="flex items-center space-x-2">
              <Checkbox
                id="merge-overlapping"
                v-model="mergeOverlapping"
              />
              <Label
                for="merge-overlapping"
                class="text-sm font-normal leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
              >
                Automatically merge overlapping periods
              </Label>
            </div>
          </div>

          <!-- Comments Section -->
          <div class="space-y-2">
            <Label for="comments">Comments</Label>
            <Textarea
              id="comments"
              v-model="comments"
              placeholder="Add any additional notes or comments..."
              class="min-h-[100px]"
            />
          </div>

          <!-- Action Buttons -->
          <div class="flex flex-col gap-4 sm:flex-row sm:justify-end">
            <Button
              variant="outline"
              class="w-full min-h-[44px] sm:w-auto"
              @click="resetForm"
            >
              Reset
            </Button>
            <Button
              :disabled="isLoading || periods.length === 0"
              class="w-full min-h-[44px] sm:w-auto"
              @click="calculateAbsences"
            >
              <span v-if="isLoading">Calculating...</span>
              <span v-else>Calculate</span>
            </Button>
          </div>

          <!-- Error Message -->
          <div v-if="errorMessage" class="rounded-lg border border-destructive bg-destructive/10 p-4">
            <p class="text-sm text-destructive">{{ errorMessage }}</p>
          </div>

          <!-- Results Section -->
          <div v-if="results.length > 0" class="space-y-4">
            <div class="border-t pt-4">
              <h3 class="mb-4 text-lg font-semibold">Results</h3>
              <div class="space-y-3">
                <div
                  v-for="(result, index) in results"
                  :key="index"
                  class="rounded-lg border p-4"
                >
                  <div class="grid grid-cols-1 gap-2 text-sm sm:grid-cols-2">
                    <div>
                      <span class="text-muted-foreground">Period:</span>
                      <span class="ml-2 font-medium">
                        {{ result.absence_start }} to {{ result.absence_end }}
                      </span>
                    </div>
                    <div>
                      <span class="text-muted-foreground">Window:</span>
                      <span class="ml-2 font-medium">
                        {{ result.window_start }} to {{ result.window_end }}
                      </span>
                    </div>
                    <div class="col-span-1 sm:col-span-2">
                      <span class="text-muted-foreground">Total Days:</span>
                      <span class="ml-2 text-lg font-bold">
                        {{ result.total_days_in_window }} days
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- Info Card -->
      <Card class="mx-auto mt-8 max-w-2xl">
        <CardHeader>
          <CardTitle>How it works</CardTitle>
          <CardDescription>
            Understanding the 365-day rolling window calculation
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-2 text-sm text-muted-foreground">
          <p>
            For each absence period, the calculator determines a 365-day window ending on the absence end date.
          </p>
          <p>
            It then counts all absence days within that window, automatically merging overlapping or adjacent periods to prevent double-counting.
          </p>
          <p>
            All calculations are performed securely in your browser using WebAssembly.
          </p>
        </CardContent>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { AbsencePeriod, CalculationResult } from '~/types/absence'

const { calculate, validate } = useAbsenceCalculator()

const periods = ref<AbsencePeriod[]>([])
const errors = ref<Record<number, string>>({})
const comments = ref('')
const mergeOverlapping = ref(true)
const isLoading = ref(false)
const errorMessage = ref('')
const results = ref<CalculationResult[]>([])

const addPeriod = () => {
  const today = new Date().toISOString().split('T')[0]
  periods.value.push({
    start_date: today,
    end_date: today
  })
}

const removePeriod = (index: number) => {
  periods.value.splice(index, 1)
  delete errors.value[index]
  // Reindex errors
  const newErrors: Record<number, string> = {}
  Object.keys(errors.value).forEach(key => {
    const numKey = Number(key)
    if (numKey > index) {
      newErrors[numKey - 1] = errors.value[numKey]
    } else if (numKey < index) {
      newErrors[numKey] = errors.value[numKey]
    }
  })
  errors.value = newErrors
  results.value = []
  errorMessage.value = ''
}

const validatePeriod = (index: number) => {
  const period = periods.value[index]
  if (!period) return

  const start = new Date(period.start_date)
  const end = new Date(period.end_date)

  if (isNaN(start.getTime())) {
    errors.value[index] = 'Invalid start date'
    return
  }

  if (isNaN(end.getTime())) {
    errors.value[index] = 'Invalid end date'
    return
  }

  if (end < start) {
    errors.value[index] = 'End date must be on or after start date'
    return
  }

  delete errors.value[index]
  errorMessage.value = ''
}

const resetForm = () => {
  periods.value = []
  errors.value = {}
  comments.value = ''
  results.value = []
  errorMessage.value = ''
}

const calculateAbsences = async () => {
  // Validate all periods
  periods.value.forEach((_, index) => {
    validatePeriod(index)
  })

  if (Object.keys(errors.value).length > 0) {
    errorMessage.value = 'Please fix validation errors before calculating'
    return
  }

  if (periods.value.length === 0) {
    errorMessage.value = 'Please add at least one absence period'
    return
  }

  isLoading.value = true
  errorMessage.value = ''
  results.value = []

  try {
    const isValid = await validate(periods.value)
    if (!isValid) {
      errorMessage.value = 'Invalid absence periods. Please check your input.'
      return
    }

    const calculatedResults = await calculate(periods.value)
    results.value = calculatedResults
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Failed to calculate absences'
  } finally {
    isLoading.value = false
  }
}

// Initialize with one period
onMounted(() => {
  addPeriod()
})
</script>
