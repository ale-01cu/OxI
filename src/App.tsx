import { useState } from "react";
import SearchViewSimple from "./components/search-view-simple";
import SearchView from "./components/search-view";



function App() {
  const [ isSimpleView, _ ] = useState(false);

  if(isSimpleView) return <SearchViewSimple />;
  return <SearchView />;
}

export default App;
