<script lang="ts" setup>
import type { PropType } from 'vue'
import Chart from 'chart.js/auto'
import zoomPlugin from 'chartjs-plugin-zoom'
import 'chartjs-adapter-moment'
import type { MarketItemOnCrack } from '../pages/all_items.vue'

const props = defineProps({
  chest: {
    type: Object as PropType<MarketItemOnCrack>,
    required: true,
  },
})

const chart_elem = ref()
let myChart: any

function set_chart_data(data: [string, number, string][]) {
  myChart.datasets[0].data = data.map(([date, price]) => ({ x: date, y: price }))
  myChart.update()
}

watch(props.chest, (chests) => {
  if (chests.values && myChart !== undefined) {
    set_chart_data(chests.values)
  }
})

Chart.register(zoomPlugin)
onMounted(() => {
  myChart = new Chart(chart_elem.value, {
    type: 'line',
    data: {
      datasets: [{
        label: 'Price',
        borderColor: '#6d28d9',
        pointRadius: 1,
        data: (props.chest.values || []).map(([date, price]) => ({ x: date, y: price })),
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
              enabled: true,
            },
            mode: 'xy',
          },
          pan: {
            enabled: true,
            mode: 'xy',
          },
        },
      },
    },
  })
})

onUnmounted(() => {
  myChart.destroy()
})
</script>

<template lang="pug">
div.flex.flex-col.justify-between.border.border-red-500.rounded-xl.p-2
  p.text-red.font-bold(v-if="chest.error") {{ chest.error }}
  div.flex.justify-between.items-center {{ chest.name }}
  div.flex.justify-center
    img(:src="'https://community.akamai.steamstatic.com/economy/image/' + chest.icon_url")
  canvas(ref="chart_elem")
</template>
