<script lang="ts" setup>
import type { PropType } from 'vue'
import Chart from 'chart.js/auto'
import type { MarketItemOnCrack } from '../pages/all_items.vue'
import 'chartjs-adapter-moment'

const props = defineProps({
  chest: {
    type: Object as PropType<MarketItemOnCrack>,
    required: true,
  },
})

const chart = ref()
let myChart: any

onMounted(() => {
  if (props.chest.values) {
    myChart = new Chart(chart.value, {
      type: 'line',
      data: {
        labels: [],
        datasets: [{
          label: 'Price',
          data: props.chest.values.map(([time, median]) => ({ x: time, y: median })),
        }],
      },
      options: {
        scales: {
          x: {
            type: 'time',
            min: Date.now() - (3 * 24 * 60 * 60 * 1000),
            max: Date.now(),
          },
        },
      },
    })
  }
})
</script>

<template lang="pug">
div.flex.flex-col.justify-between.border.border-rose-500.rounded-xl.p-2
  p.text-red.font-bold(v-if="chest.error") {{ chest.error }}
  div.flex.justify-between.items-center {{ chest.name }}
  div.flex.justify-center
    img(:src="'https://community.akamai.steamstatic.com/economy/image/' + chest.icon_url")
  canvas(ref="chart")
</template>
