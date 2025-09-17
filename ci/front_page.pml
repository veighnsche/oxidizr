# QA Evidence for {{ commit }}
<grid cols=3 gap=16>
  <card title="Tests">
    <artifact.summary id="tests-summary" />
    [[a:tests-summary | Full Summary]]
  </card>
  <card title="Coverage">
    <artifact.table id="coverage" />
  </card>
  <card title="Failures">
    <artifact.markdown id="failures" />
  </card>
</grid>
