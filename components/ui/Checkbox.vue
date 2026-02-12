<template>
  <input
    type="checkbox"
    :class="cn(
      'peer h-4 w-4 shrink-0 rounded-sm border border-primary ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
      modelValue && 'bg-primary text-primary-foreground',
      $attrs.class
    )"
    :checked="modelValue"
    @change="$emit('update:modelValue', ($event.target as HTMLInputElement).checked)"
    v-bind="$attrs"
  />
</template>

<script setup lang="ts">
import { cn } from '@/lib/utils'

interface Props {
  modelValue?: boolean
}

defineProps<Props>()
defineEmits<{
  'update:modelValue': [value: boolean]
}>()

defineOptions({
  inheritAttrs: false
})
</script>
