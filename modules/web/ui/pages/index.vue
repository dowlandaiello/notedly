<template>
  <section class="section">
    <div class="columns is-mobile">
      <div class="column card is-2">
        <div class="container has-text-centered">
          <b-icon icon="plus" class="card-icon" size="is-medium" />
          <br>
          Make something new!
        </div>
      </div>
      <div class="column card is-2">
        <div class="container has-text-centered">
          <b-icon icon="folder" class="card-icon" size="is-medium" />
          <br>
          Ungrouped Notes
        </div>
      </div>
      <div class="column card is-2">
        <div class="container has-text-centered">
          <b-icon icon="folder" class="card-icon" size="is-medium" />
          <br>
          Ungrouped Notes
        </div>
      </div>
    </div>
  </section>
</template>

<script>
export default {
  name: 'HomePage',
}
</script>

<style scoped>
.card {
  display: flex;
  align-items: center;
  text-align: center;
}

.card {
  height: 16rem;
  
  margin-left: 1.5rem;
  margin-right: 1.5rem;
}
</style>
