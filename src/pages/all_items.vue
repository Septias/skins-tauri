<script lang="ts" setup>
import { invoke } from '@tauri-apps/api'
import type { MarketItem } from 'src-tauri/bindings/MarketItem'

const error = ref('')
const chests: Ref<MarketItemOnCrack[]> = useStorage('all_items', [])

interface MarketItemOnCrack extends MarketItem {
  error?: string
}

async function get_all_containers() {
  console.log('updating all containers')

  try {
    const res: any = await invoke('get_all_csgo_containers')
    console.log(res)

    chests.value = (Object.values(res) as MarketItemOnCrack[]).sort((a, b) => a.classid - b.classid)
  }
  catch (err) {
    console.error('problem requesting chests', err)
    error.value = err as string
  }
}

get_all_containers()
</script>

<template lang="pug">
button.btn(@click="get_all_containers") Get all containers
div.c-grid.p-10
  p {{ error }}
  div
    h1.text-2xl.font-bold.text-center.text-rose-500.mb-10 CS:GO Chest Statistics
div.chest-grid
  div.flex.flex-col.justify-between.border.border-rose-500.rounded-xl.p-2.shadow-xl(v-for="chest in chests" :key="chest.classid")
    p.text-red.font-bold(v-if="chest.error") {{ chest.error }}
    div.flex.justify-between.items-center {{ chest.name }}
    div.flex.justify-center
      img(:src="'https://community.akamai.steamstatic.com/economy/image/' + chest.icon_url")
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
