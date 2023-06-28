<script lang="ts" setup>
import { invoke } from '@tauri-apps/api'
import type { MarketItem } from 'src-tauri/bindings/MarketItem'
import type {PriceHistoryResponse} from 'src-tauri/bindings/PriceHistoryResponse'

const error = ref('')
const chests: Ref<MarketItemOnCrack[]> = useStorage('all_items', [])

export interface MarketItemOnCrack extends MarketItem {
  error?: string
  values?: [string, number, string][]
}

async function get_price_history(assets: Record<string, any>[]): Promise<Record<number, {Ok: PriceHistoryResponse} | {Error: string}>[]> {
  return await invoke('get_asset_price_history', { assets })
}

async function get_all_containers() {
  try {
    const res: any = await invoke('get_all_csgo_basic_cases')
    const prices = await get_price_history(Object.entries(res).map(([classid, item]) => [Number(classid), item.name]))
    console.log(prices);

    for (const [id, value] of Object.entries(prices)) {
      console.log(id, value);

      if (value.Ok) {
        res[id].values = value.Ok
      } else {
        res[id].error = value.Error
      }
    }
    chests.value = (Object.values(res) as MarketItemOnCrack[]).sort((a, b) => a.classid - b.classid)
  }
  catch (err) {
    console.error('problem requesting chests', err)
    error.value = err as string
  }
}
</script>

<template lang="pug">
div.p-5
  button.btn(@click="get_all_containers") Get all containers
  //p.text-center.text-red-800.font-bold {{ error }}
  h1.text-2xl.font-bold.text-center.text-rose-500.mb-5 CS:GO Chest Statistics
  div.chest-grid
    chest(v-for="chest in chests" :key="chest.classid" :chest="chest")

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
