import { useMemo, useState } from "react";

const now = () => new Date().toISOString();

export default function App() {
  const [activities, setActivities] = useState([]);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [error, setError] = useState("");

  const sortedActivities = useMemo(
    () => [...activities].sort((a, b) => a.name.localeCompare(b.name)),
    [activities]
  );

  const addActivity = (e) => {
    e.preventDefault();
    const cleanName = name.trim();
    const cleanDescription = description.trim();

    if (!cleanName) {
      setError("Activity name is required.");
      return;
    }

    const exists = activities.some(
      (activity) => activity.name.toLowerCase() === cleanName.toLowerCase()
    );

    if (exists) {
      setError("Activity already exists.");
      return;
    }

    setActivities((prev) => [
      ...prev,
      {
        id: crypto.randomUUID(),
        name: cleanName,
        description: cleanDescription || "No description provided.",
        state: "OFF",
        history: []
      }
    ]);

    setName("");
    setDescription("");
    setError("");
  };

  const updateState = (id, nextState) => {
    setActivities((prev) =>
      prev.map((activity) => {
        if (activity.id !== id || activity.state === nextState) {
          return activity;
        }

        return {
          ...activity,
          state: nextState,
          history: [...activity.history, `${now()} | turned ${nextState}`]
        };
      })
    );
  };

  return (
    <main className="page">
      <section className="panel">
        <h1>AI Agent Activity Interface</h1>
        <p>Turn AI agent activities on and off and inspect each activity timeline.</p>

        <form className="form" onSubmit={addActivity}>
          <label>
            Activity name
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. Web Search"
            />
          </label>

          <label>
            Description
            <input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="What this activity does"
            />
          </label>

          <button type="submit">Add activity</button>
        </form>

        {error && <p className="error">{error}</p>}
      </section>

      <section className="panel">
        <h2>Activities</h2>
        {sortedActivities.length === 0 ? (
          <p>No activities created yet.</p>
        ) : (
          <ul className="activity-list">
            {sortedActivities.map((activity) => (
              <li key={activity.id} className="activity-card">
                <div className="activity-header">
                  <div>
                    <h3>{activity.name}</h3>
                    <p>{activity.description}</p>
                  </div>
                  <span className={`badge ${activity.state.toLowerCase()}`}>{activity.state}</span>
                </div>

                <div className="actions">
                  <button
                    type="button"
                    onClick={() => updateState(activity.id, "ON")}
                    disabled={activity.state === "ON"}
                  >
                    Turn ON
                  </button>
                  <button
                    type="button"
                    className="secondary"
                    onClick={() => updateState(activity.id, "OFF")}
                    disabled={activity.state === "OFF"}
                  >
                    Turn OFF
                  </button>
                </div>

                <details>
                  <summary>History ({activity.history.length})</summary>
                  {activity.history.length === 0 ? (
                    <p>No history yet.</p>
                  ) : (
                    <ul className="history">
                      {activity.history.map((entry, index) => (
                        <li key={`${activity.id}-h-${index}`}>{entry}</li>
                      ))}
                    </ul>
                  )}
                </details>
              </li>
            ))}
          </ul>
        )}
      </section>
    </main>
  );
}
