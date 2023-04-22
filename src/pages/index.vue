<script lang="ts" setup>
import { invoke } from '@tauri-apps/api'
import { ref } from 'vue'

const disp = ref('ja')
invoke('get_user_containers').then((res) => {
  disp.value = res
  console.log(res)
})
</script>

<template lang="pug">
p.p-2 hi
p ja: {{ disp }}
</template>
