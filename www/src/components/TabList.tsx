import {
  Children,
  isValidElement,
  ReactElement,
  ReactNode,
  useState,
} from "react";
import TabItem from "./TabItem";

const TabList = ({
  children,
  activeTabIndex = 0,
}: {
  children: ReactNode;
  activeTabIndex: number;
}) => {
  const [activeTab, setActiveTab] = useState(activeTabIndex);

  const handleTabClick = (index: number) => {
    setActiveTab(index);
  };

  const tabs = Children.toArray(children).filter(
    (child): child is ReactElement<{ children: ReactNode; label: string }> =>
      isValidElement(child) && child.type === TabItem,
  );

  return (
    <div className="flex max-h-full w-full flex-1 flex-col items-center">
      <nav className="flex min-h-12 w-full bg-zinc-50 text-zinc-800">
        {tabs.map((tab, index) => (
          <button
            key={`tab-btn-${index}`}
            onClick={() => handleTabClick(index)}
            className={`m-1 border-2 border-zinc-800 bg-zinc-100 px-2 ${activeTab === index && "bg-zinc-800 text-zinc-100"}`}
          >
            {tab.props.label}
          </button>
        ))}
      </nav>
      {tabs[activeTab]}
    </div>
  );
};

export default TabList;
