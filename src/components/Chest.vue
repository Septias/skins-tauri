<script lang="ts" setup>
import type { PropType } from 'vue'
import Chart from 'chart.js/auto'
import type { MarketItemOnCrack } from '../pages/all_items.vue'
import 'chartjs-adapter-moment'
import zoomPlugin from 'chartjs-plugin-zoom';

const props = defineProps({
  chest: {
    type: Object as PropType<MarketItemOnCrack>,
    required: true,
  },
})

const chart = ref()
let myChart: any

Chart.register(zoomPlugin)

onMounted(() => {
  if (props.chest.values) {
    const data = props.chest.values.map(([time, median]) => ({ x: new Date(time), y: median }))
    myChart = new Chart(chart.value, {
      type: 'line',
      data: {
        labels: [],
        datasets: [{
          label: 'Price',
          data,
          tension: 0.1,
        }],
      },
      options: {
        scales: {
          x: {
            type: 'time',
          },
        },
        plugins: {
          zoom: {
            zoom: {
              wheel: {
                enabled: true,
                // modifierKey: 'shift',
              },
              pinch: {
                enabled: true
              },
              mode: 'xy',
            },
            pan: {
              enabled: true,
              mode: 'xy',
            }
          }
        }
      },
    })
  }
})
onUnmounted(() => {
  myChart.destroy()
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
