package strangelove

import (
	"testing"

	"github.com/stretchr/testify/require"
)

// Tests that a note module may not handshake with another note
// module.
//
// TODO: Test that a valid connection can be created after an invalid
// one is attempted.
func TestNoteNoteHandshake(t *testing.T) {
	suite := NewSuite(t)
	_, _, err := suite.CreateChannel(suite.ChainA.Note, suite.ChainB.Note, &suite.ChainA, &suite.ChainB)
	require.ErrorContains(t, err, "no new channels created", "note <-/-> note")

	channels := suite.QueryChannelsInState(&suite.ChainB, CHANNEL_STATE_TRY)
	require.Len(t, channels, 1, "try note stops in first step")
	channels = suite.QueryChannelsInState(&suite.ChainB, CHANNEL_STATE_INIT)
	require.Len(t, channels, 1, "init note doesn't advance")
}
