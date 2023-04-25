<script lang="ts" setup>
import { invoke } from '@tauri-apps/api'
import type { FullAsset } from 'src-tauri/bindings/FullAsset'
import type { MarketPrice } from 'src-tauri/bindings/MarketPrice'

const user_input = useStorage('user_id', '')
const error = ref('')
const chests: Ref<FullAsset[]> = useStorage('chests', [])
const prices: Ref<{ [index: number]: MarketPrice }> = useStorage('prices', {})

function update() {
  invoke('get_user_containers', { game: 730, user: user_input.value }).then((res) => {
    chests.value = res as FullAsset[]
    console.log(res)
  }).catch((err) => {
    error.value = err
  })
  invoke('get_asset_prices', { assets: chests.value.map(chest => chest.classid) }).then((res) => {
    console.log(res)
    prices.value = res as { [index: number]: MarketPrice }
  }).catch((err) => {
    error.value = err
  })
}

// const total_value = chests.value.map(chest => chest.amount * (prices.value[chest.classid].median_price || 0)).reduce((a, b) => a + b, 0).toFixed(2)
</script>

<template lang="pug">
div.c-grid.p-10
  div
    h1.text-2xl.font-bold.text-center.text-rose-500.mb-10 CS:GO Chest Value
    div.flex.justify-between.items-center.gap-2
      div.flex.gap-2
        input.rounded.p-2.border.leading-none(v-model="user_input" alt="User ID" placeholder="User ID")
        button.rounded.bg-rose-500.p-2.leading-none(@click="update") Update
      //div Total chest value: {{ total_value }}€
    div.chest-grid
      div.border.border-rose-500.rounded-xl.p-2.shadow-xl(v-for="chest in chests")
        h1.text-xl.font-bold {{ chest.amount }} x {{ chest.name }}
        div.flex.justify-center
          img(:src="'https://community.akamai.steamstatic.com/economy/image/' + chest.icon_url")
        p(v-if="prices.value[chest.classid]") {{ (chest.amount * chest.price.median_price).toFixed(2) }}€
</template>

<style lang="sass">
.chest-grid
  display: grid
  grid-template-columns: repeat(auto-fill, minmax(400px, 1fr))
  grid-gap: 1rem
  padding: 1rem

.c-grid
  display: grid
  @apply items-center justify-stretch

@media screen and (min-width: 450px)
  .min-width
    width: 450px

  .c-grid
    @apply justify-center
</style>
