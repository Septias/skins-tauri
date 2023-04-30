<script lang="ts" setup>
import { invoke } from '@tauri-apps/api'
import type { FullAsset } from 'src-tauri/bindings/FullAsset'
import type { MarketPrice } from 'src-tauri/bindings/MarketPrice'
import { open } from '@tauri-apps/api/shell'

interface MarketPriceOnCrack extends MarketPrice {
  error?: string
}

interface UpdatePriceResponse {
  [index: number]: { Ok: MarketPrice | undefined; Err: string | undefined }
}

const user_input = useStorage('user_id', '')
const error = ref('')
const chests: Ref<FullAsset[]> = useStorage('chests', [])
const prices = reactive({} as { [index: number]: MarketPriceOnCrack })

async function update_inv() {
  try {
    const res = await invoke('get_user_containers', { game: 730, user: user_input.value })
    chests.value = (res as FullAsset[]).sort((a, b) => a.classid - b.classid)
    console.log(res)
  }

  catch (err) {
    console.log('problem requesting chests', err)
    error.value = err as string
  }
}

async function update_prices() {
  try {
    const res: UpdatePriceResponse = await invoke('get_asset_prices', { assets: chests.value.map(chest => [chest.classid, chest.market_hash_name]) })
    for (const chest_id in res) {
      const marketprice = res[chest_id].Ok
      if (marketprice) {
        prices[chest_id] = marketprice
      }
      else {
        console.log(res[chest_id])
        prices[chest_id].error = res[chest_id].Err
      }
    }
  }
  catch (err) {
    console.log('problem requesting prices', err)
    error.value = err as string
  }
}

async function multisell(market_hash_name: string) {
  open(`https://steamcommunity.com/market/multisell?appid=730&contextid=2&items%5B%5D=${market_hash_name}`)
}

update_prices()

const total_value = computed(() => chests.value.map(chest => chest.amount * (prices[chest.classid]?.median_price || 0)).reduce((a, b) => a + b, 0).toFixed(6))
</script>

<template lang="pug">
div.c-grid.p-10
  p {{ error }}
  p hi
  div
    h1.text-2xl.font-bold.text-center.text-rose-500.mb-10 CS:GO Chest Value
    div.flex.justify-between.items-center.gap-2
      div.flex.gap-2
        input.rounded.p-2.border.leading-none(v-model="user_input" alt="User ID" placeholder="User ID")
        button.btn(@click="update_inv") Update Inventory
        button.btn(@click="update_prices") Update Prices
      div Total chest value: {{ total_value }}€
div.chest-grid
  div.flex.flex-col.justify-between.border.border-rose-500.rounded-xl.p-2.shadow-xl(v-for="chest in chests" :key="chest.classid")
    p.text-red.font-bold(v-if="prices[chest.classid] && prices[chest.classid].error") {{ prices[chest.classid].error }}
    div.flex.justify-between.items-center
      h1.text-xl.font-bold {{ chest.amount  }} x {{ chest.name }}
      p.whitespace-nowrap.font-bold(v-if="prices[chest.classid]") @ {{ prices[chest.classid].median_price.toFixed(2) }}
    div.flex.justify-center
      img(:src="'https://community.akamai.steamstatic.com/economy/image/' + chest.icon_url")

    div.flex.justify-between
      button.btn(@click="() => multisell(chest.market_hash_name)") Sell
      p.text-right.font-bold(v-if="prices[chest.classid]") total value: {{(chest.amount * prices[chest.classid].median_price).toFixed(2)}}
.absolute.top-0.right-0.p-2.btn
  router-link(to="/all_items") All Items
</template>

<style lang="sass">
.chest-grid
  display: grid
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr))
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
